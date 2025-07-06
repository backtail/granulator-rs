use core::sync::atomic::{AtomicBool, Ordering::Relaxed};

// data management
use heapless::Vec;

// randomness
use oorandom::Rand32;

// math
use micromath::F32Ext;

// scheduler specific
use super::scheduler::Scheduler;
use core::time::Duration;

// crate specific
use crate::grains_vector::GRAINS;
use crate::pointer_wrapper::BufferSlice;
use crate::source::Source;
use crate::statistics::*;
use crate::user_settings::GranulatorParameter::*;
use crate::window_function::WindowFunction;

// audio processing
use super::audio_tools::soft_clip;

/// The most grains grains that can possibly play at the same time.
///
/// Increasing this will increase processing demands. Only changeable if one recompiles the
/// crate with a different number. This will change in the future.
pub const MAX_GRAINS: usize = 50;

/// Smallest value at which the spreading algorithm should be activated
///
/// The ADC of the Electrosmith Daisy Seed (STM32h750) has a resolultion of 12 bit, so the
/// smallest number that can be represented is 1/(2^12-1) = 0.00024420024. To give a little
/// room for error, this value is being chosen to be ten times bigger.
const SPREAD_ESPILON: f32 = 0.0024420024;

const MIN_GRAIN_SIZE_IN_SAMPLES: f32 = 480.0;
const MAX_GRAIN_SIZE_IN_MS: f32 = 2000.0;
const MAX_DELAY_IN_MS: f32 = 3000.0;
const MAX_PITCH_RANGE: f32 = 2.0;
const MASTER_VOLUME_RANGE: f32 = 5.0;

static BUFFER_IS_LOADED: AtomicBool = AtomicBool::new(false);

pub fn is_buffer_loaded() -> bool {
    BUFFER_IS_LOADED.load(Relaxed)
}

// ==============
// AUDIO CALLBACK
// ==============

pub fn get_next_sample() -> f32 {
    if is_buffer_loaded() {
        unsafe {
            soft_clip(
                GRAINS.get_next_sample() * MasterVolume.get_unit_float() * MASTER_VOLUME_RANGE,
            )
        }
    } else {
        0.0
    }
}

/// The brain of the granular synthesis algorithm.
#[derive(Debug)]
pub struct Granulator {
    scheduler: Scheduler,
    // grains: &mut GrainsVector,
    audio_buffer: Option<BufferSlice>, // points to the beginning of the buffer
    buffer_length: Option<f32>,

    // parameter bounds
    max_grain_size_in_ms: f32,

    // misc
    current_id_counter: usize,
    fs: usize,

    // RNG
    rng: Rand32,
}

impl Granulator {
    /**
    Constructs the Granulator object. A sample frequency is required, which can be changed
    during playback if wanted.

    ## Example

    ```
    let g = granulator::Granulator::new(48_000);
    ```
    */
    pub fn new(fs: usize) -> Self {
        // The seed of the of the PRNG is being determined by the derefence of the `seed` argument.
        // This results in a non-repeating sequence of random numbers every time the the program gets
        // restarted. No need to generate a new random seed.
        let random_seed = 0;
        let random_memory_location = core::ptr::addr_of!(random_seed);

        Granulator {
            scheduler: Scheduler::new(),
            // grains: GrainsVector::new(),
            audio_buffer: None,
            buffer_length: None,

            max_grain_size_in_ms: 1000.0,

            current_id_counter: 0,
            fs,

            rng: Rand32::new(random_memory_location as u64),
        }
    }

    // ==========================
    // SETTER WITH BOUND CHECKING
    // ==========================

    /// Sets the sample rate of the `Granulator` for calculations. This should be updated as soon
    /// as your sound driver changes its sample rate!
    pub fn set_sample_rate(&mut self, fs: usize) -> Result<(), usize> {
        if fs > 8_000 && fs < 192_000 {
            self.fs = fs;
            Ok(())
        } else {
            Err(fs)
        }
    }

    /// Sets the maximum value the grain size parameter can reach. Is in between 100ms and the
    /// length of the currently loaded sample. This value is in ms.
    ///
    /// May be updated frequently or just once in the beginning.
    pub fn set_max_grain_size(&mut self, grain_size_in_ms: f32) {
        if self.audio_buffer.is_some() {
            let max_grain_size = (self.buffer_length.unwrap() / self.fs as f32) * 1000.0;
            let min_grain_size = 100.0;

            if grain_size_in_ms < min_grain_size {
                self.max_grain_size_in_ms = min_grain_size;
            }
            if grain_size_in_ms > max_grain_size {
                self.max_grain_size_in_ms = max_grain_size;
            }
            if grain_size_in_ms >= min_grain_size && grain_size_in_ms <= max_grain_size {
                self.max_grain_size_in_ms = grain_size_in_ms;
            }
        }
    }

    // ========================
    // AUDIO BUFFER INTERACTION
    // ========================

    /// Sets a new audio buffer for the algorithm to work on.
    pub fn set_audio_buffer(&mut self, buffer: &[f32]) {
        // create slice buffer
        self.audio_buffer = Some(BufferSlice::from_slice(buffer));
        self.buffer_length = Some(self.audio_buffer.as_ref().unwrap().length);
        BUFFER_IS_LOADED.store(true, Relaxed);
    }

    /// Removes current audio buffer and stops the algorithm
    pub fn remove_audio_buffer(&mut self) {
        self.audio_buffer = None;
        self.buffer_length = None;
        BUFFER_IS_LOADED.store(false, Relaxed);
    }

    // ====================
    // SCHEDULER MANAGEMENT
    // ====================

    /// Updates the internal scheduler which keeps track of which grain will be started/triggered
    /// at which point in time. It also removes every grain, that has finished playing.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    /// Preferrably at the end.
    pub fn update_scheduler(&mut self, time_step: Duration) {
        if self.audio_buffer.is_some() {
            self.spawn_future_grains();
            let ids = self.scheduler.update_clock(time_step);
            self.activate_grains(&ids);
            self.remove_finished_grains();
        }
    }

    fn remove_finished_grains(&mut self) {
        // collect all finished grain ids
        let mut remove_ids: Vec<usize, MAX_GRAINS> = Vec::new();
        unsafe {
            for grain in GRAINS.get_mut_grains() {
                if grain.finished {
                    remove_ids.push(grain.id).unwrap();
                }
            }
        }

        // remove all finished active grains
        for id in remove_ids {
            self.scheduler.remove_grain(id).unwrap();
            unsafe {
                GRAINS.remove_grain(id).unwrap();
            }
        }
    }

    fn activate_grains(&mut self, ids: &Vec<usize, MAX_GRAINS>) {
        if self.audio_buffer.is_some() {
            for id in ids {
                let velocity = self.get_new_velocity();
                let pitch = self.get_new_pitch();
                let offset = self.get_new_offset();
                let grain_size = self.get_new_grain_size();
                unsafe {
                    GRAINS
                        .push_grain(
                            *id,
                            self.audio_buffer
                                .as_ref()
                                .unwrap()
                                .get_sub_slice(offset, grain_size),
                            self.get_new_window(),
                            self.get_new_source(),
                            pitch,
                            velocity,
                        )
                        .unwrap();
                }
            }
        }
    }

    fn spawn_future_grains(&mut self) {
        // the difference between all future grains and number of active grains should be spawned, but never less than zero
        let to_be_spawned = self
            .get_active_grains()
            .checked_sub(self.scheduler.future_vector.len())
            .unwrap_or(0);

        // spawn future grains
        for _ in 0..to_be_spawned {
            let id = self.get_new_id();
            let delay = self.get_new_delay();
            self.scheduler.schedule_grain(id, delay).ok();
        }
    }

    fn get_new_id(&mut self) -> usize {
        let current_id = self.current_id_counter;
        if self.current_id_counter >= usize::MAX - 1 {
            self.current_id_counter = 0;
        }
        self.current_id_counter += 1;

        current_id
    }

    // ==============================
    // PARAMETER RUNTIME CALCULATIONS
    // ==============================

    fn get_active_grains(&self) -> usize {
        (ActiveGrains.get_unit_float() * MAX_GRAINS as f32) as usize
    }

    fn get_new_offset(&mut self) -> usize {
        if OffsetSpread.get_unit_float() >= SPREAD_ESPILON {
            self.get_offset_spread()
        } else {
            self.get_offset() as usize
        }
    }

    fn get_new_grain_size(&mut self) -> f32 {
        if GrainSizeSpread.get_unit_float() >= SPREAD_ESPILON {
            self.get_grain_size_spreaded()
        } else {
            self.get_grain_size_in_samples()
        }
    }

    fn get_new_window(&self) -> WindowFunction {
        WindowFunction::Sine
    }

    fn get_new_source(&self) -> Source {
        Source::AudioFile
    }

    fn get_new_pitch(&mut self) -> f32 {
        let new_pitch;
        if PitchSpread.get_unit_float() >= SPREAD_ESPILON {
            new_pitch = self.get_pitch_spreaded()
        } else {
            new_pitch = self.get_pitch()
        }

        10.0.powf(new_pitch * 2.0 - 1.0) * MAX_PITCH_RANGE
    }

    fn get_new_delay(&mut self) -> Duration {
        if DelaySpread.get_unit_float() >= SPREAD_ESPILON {
            self.get_delay_spreaded()
        } else {
            self.get_delay()
        }
    }

    fn get_new_velocity(&mut self) -> f32 {
        if VelocitySpread.get_unit_float() >= SPREAD_ESPILON {
            self.get_velocity_spreaded()
        } else {
            self.get_velocity()
        }
    }

    // ================
    // HELPER FUNCTIONS
    // ================

    /// Will be calculated as: `[0;1]` * (`range` - `min_grain_size`)
    fn get_offset(&self) -> f32 {
        Offset.get_unit_float() * (self.buffer_length.unwrap() - MIN_GRAIN_SIZE_IN_SAMPLES)
    }

    fn get_offset_spread(&mut self) -> usize {
        let range = self.get_offset();

        // Calculate random offset
        let variance =
            OffsetSpread.get_unit_float() * get_random_bipolar_float(&mut self.rng) * range;

        // Add variance to offset
        let signed_offset = range + variance;

        // Clamp value
        signed_offset.clamp(0.0, range) as usize
    }

    fn get_grain_size_in_samples(&self) -> f32 {
        // Convert ms into samples
        ms_to_samples(
            self.fs as f32,
            GrainSize.get_unit_float() * MAX_GRAIN_SIZE_IN_MS,
        )
        // Prevent over and under shoot
        .clamp(
            MIN_GRAIN_SIZE_IN_SAMPLES,
            self.buffer_length.unwrap() - self.get_offset(), // should be actual offset with possible variance
        )
    }

    fn get_grain_size_spreaded(&mut self) -> f32 {
        // Calculate random offset
        let variance = GrainSizeSpread.get_unit_float()
            * get_random_bipolar_float(&mut self.rng)
            * self.buffer_length.unwrap();

        // Add variance
        let signed_grain_size = self.get_grain_size_in_samples() as f32 + variance;

        // Clamp value
        signed_grain_size.clamp(
            MIN_GRAIN_SIZE_IN_SAMPLES,
            self.buffer_length.unwrap() - self.get_offset(), // should be actual offset with possible variance
        )
    }

    fn get_pitch(&self) -> f32 {
        Pitch.get_unit_float()
    }

    fn get_pitch_spreaded(&mut self) -> f32 {
        // Calculate random offset
        let variance = PitchSpread.get_unit_float() * get_random_bipolar_float(&mut self.rng);

        // Add variance
        let signed_grain_size = self.get_pitch() + variance;

        // Clamp value
        signed_grain_size.clamp(0.0, 1.0)
    }

    fn get_delay(&self) -> Duration {
        Duration::from_millis((Delay.get_unit_float() * MAX_DELAY_IN_MS) as u64)
    }

    fn get_delay_spreaded(&mut self) -> Duration {
        // Calculate random offset
        let variance = DelaySpread.get_unit_float()
            * get_random_unipolar_float(&mut self.rng)
            * MAX_DELAY_IN_MS;

        // Add variance
        self.get_delay() + Duration::from_millis(variance as u64)
    }

    fn get_velocity(&self) -> f32 {
        Velocity.get_unit_float()
    }

    fn get_velocity_spreaded(&mut self) -> f32 {
        // Calculate random offset
        let variance = VelocitySpread.get_unit_float() * get_random_bipolar_float(&mut self.rng);

        // Add variance
        let signed_velocity = self.get_velocity() + variance;

        // Clamp value
        signed_velocity.clamp(0.0, 1.0)
    }
}

fn ms_to_samples(fs: f32, value_in_ms: f32) -> f32 {
    (fs / 1000.0) * value_in_ms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_settings::GranulatorParameter::{ActiveGrains, GrainSize};

    const FS: usize = 48_000;

    #[test]
    fn get_a_new_index() {
        let mut m = Granulator::new(FS);
        assert!(m.current_id_counter == 0);

        for i in 0..MAX_GRAINS {
            let new_id = m.get_new_id();

            assert!(new_id == i);
        }
    }

    #[test]
    fn activate_all_grains() {
        unsafe {
            let mut m = Granulator::new(FS);
            let buffer = [0_f32; MIN_GRAIN_SIZE_IN_SAMPLES as usize];
            GRAINS.flush();

            m.set_audio_buffer(&buffer);

            let mut ids = Vec::new();

            assert!(GRAINS.get_grains().len() == 0);

            for _ in 0..MAX_GRAINS {
                ids.push(m.get_new_id()).unwrap();
            }

            m.activate_grains(&ids);

            assert!(GRAINS.get_grains().len() == MAX_GRAINS);
        }
    }

    #[test]
    fn spawn_all_grains() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; MIN_GRAIN_SIZE_IN_SAMPLES as usize];
        m.set_audio_buffer(&buffer);
        ActiveGrains.update(1.0);

        m.spawn_future_grains();

        assert!(m.scheduler.future_vector.len() == MAX_GRAINS);
        ActiveGrains.update(0.0);
    }

    #[test]
    fn remove_all_grains() {
        unsafe {
            // setup unit test
            let mut m = Granulator::new(FS);
            let buffer = [1_f32; 10000];
            GRAINS.flush();

            let mut check_slice: Vec<usize, MAX_GRAINS> = Vec::new();
            let mut zero_slice: Vec<usize, MAX_GRAINS> = Vec::new();
            for i in 0..MAX_GRAINS {
                check_slice.push(i).unwrap();
                zero_slice.push(0).unwrap();
            }

            m.set_audio_buffer(&buffer);
            ActiveGrains.update(1.0);
            GrainSize.update(0.0);

            // update scheduler
            m.spawn_future_grains();
            let ids = m.scheduler.update_clock(Duration::from_millis(20));

            assert!(ids == check_slice);

            m.activate_grains(&ids);
            m.remove_finished_grains();

            assert!(GRAINS.get_grains().len() == MAX_GRAINS);
            assert!(m.scheduler.future_vector.len() == MAX_GRAINS);

            // finish all grains
            for _ in 0..MIN_GRAIN_SIZE_IN_SAMPLES as usize + 1 {
                GRAINS.get_next_sample();
            }

            // update schedular
            m.spawn_future_grains();
            let ids = m.scheduler.update_clock(Duration::from_millis(20));
            assert!(ids == Vec::<usize, MAX_GRAINS>::new());
            m.activate_grains(&ids);
            m.remove_finished_grains();

            assert!(GRAINS.get_grains().len() == 0);
            assert!(m.scheduler.future_vector.len() == 0);

            // next cycle

            m.spawn_future_grains();
            assert!(GRAINS.get_grains().len() == 0);
            assert!(m.scheduler.future_vector.len() == MAX_GRAINS);

            let ids = m.scheduler.update_clock(Duration::from_millis(20));
            m.activate_grains(&ids);
            assert!(GRAINS.get_grains().len() == MAX_GRAINS);
            assert!(m.scheduler.future_vector.len() == MAX_GRAINS);
        }
    }

    #[test]
    fn set_a_grain_size() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; (FS as usize / 1000) * MAX_GRAIN_SIZE_IN_MS as usize];
        m.set_audio_buffer(&buffer);

        GrainSize.update(0.0);
        assert!(m.get_grain_size_in_samples() == MIN_GRAIN_SIZE_IN_SAMPLES);

        GrainSize.update(1.0);
        assert!(m.get_grain_size_in_samples() == ms_to_samples(FS as f32, MAX_GRAIN_SIZE_IN_MS));
    }

    #[test]
    fn set_a_sample_rate() {
        let mut m = Granulator::new(FS);

        let result = m.set_sample_rate(1_000);
        assert!(m.fs == 48_000);
        assert!(result.is_err());

        let result = m.set_sample_rate(300_000);
        assert!(m.fs == 48_000);
        assert!(result.is_err());

        let result = m.set_sample_rate(44_100);
        assert!(m.fs == 44_100);
        assert!(result.is_ok());
    }
}
