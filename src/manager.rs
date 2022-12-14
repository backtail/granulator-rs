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
use crate::grains_vector::GrainsVector;
use crate::manager::GranulatorParameter::*;
use crate::pointer_wrapper::BufferSlice;
use crate::source::Source;
use crate::statistics::*;
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

/// The brain of the granular synthesis algorithm.
#[derive(Debug)]
pub struct Granulator {
    scheduler: Scheduler,
    grains: GrainsVector,
    audio_buffer: Option<BufferSlice>, // points to the beginning of the buffer

    // parameters
    master_volume: f32,
    active_grains: usize,
    offset: usize,
    grain_size_in_samples: usize,
    pitch: f32,
    delay: Duration,
    velocity: f32,

    // parameter bounds
    max_grain_size_in_ms: f32,

    // spread parameters
    sp_offset: f32,
    sp_grain_size: f32,
    sp_pitch: f32,
    sp_delay: f32,
    sp_velocity: f32,

    // current random value
    random_offset_value: usize,
    random_grain_size_value: usize,
    random_pitch_value: f32,
    random_delay_value: Duration,
    random_velocity_value: f32,

    // misc
    current_id_counter: usize,
    fs: usize,

    // RNG
    rng: Rand32,
}

/// Defines all configurable parameters
pub enum GranulatorParameter {
    /**
    Software master volume. Since granular synthesis experiences high dynamic ranges, the user
    needs to have the possibility to change the output volume. It softclips if the gain is too high.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::MasterVolume, 0.0); // master_volume = 0.0
    g.set_parameter(GranulatorParameter::MasterVolume, 1.0); // master_volume = 1.0
    ```
    */
    MasterVolume,
    /**
    Number of active grains. Currently upper limited to a const `MAX_GRAINS`. Different API coming in the future.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    // MAX_GRAINS not configurable
    g.set_parameter(GranulatorParameter::ActiveGrains, 0.0); // grain_amount = 0
    g.set_parameter(GranulatorParameter::ActiveGrains, 1.0); // grain_amount = MAX_GRAINS
    ```
    */
    ActiveGrains,

    /**
    Offset a grain has relative to the start of the audio source.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Offset, 0.0); // offset = 0
    g.set_parameter(GranulatorParameter::Offset, 1.0); // offset = source_length
    ```
    */
    Offset,
    /**
    The grain's size. May be as big as the audio sources length. Currently lower bounded to 1ms. If a grain has a
    `source_length - offset < grain_size`, its size is gonna be reduced to `source_length - offset` to prevent
    out of bounds stepping.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::GrainSize, 0.0); // grain_size = 1ms
    g.set_parameter(GranulatorParameter::GrainSize, 1.0); // grain_size = source_length - offset
    ```
    */
    GrainSize,
    /**
    The grain's playback speed. For `pitch < 1`, the playback speed is gonna slower than original. Likewise, for
    `pitch > 1` the playback speed is gonna be faster than original.

    ### Example

    Internal exponentiation by power of 10.

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Pitch, 0.0); // pitch = 0.1
    g.set_parameter(GranulatorParameter::Pitch, 0.5); // pitch = 1.0
    g.set_parameter(GranulatorParameter::Pitch, 1.0); // pitch = 10.0
    ```
    */
    Pitch,
    /**
    The grain's delay. A grain can be fired with delay. Currently limited to 1s.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Delay, 0.0); // delay = 0s
    g.set_parameter(GranulatorParameter::Delay, 1.0); // delay = 1s
    ```
    */
    Delay,
    /**
    The grain's velocity. A per grain specific gain, reminiscent of MIDI velocity.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::Velocity, 0.0); // grain_volume = 0.0
    g.set_parameter(GranulatorParameter::Velocity, 1.0); // grain_volume = 1.0
    ```
    */
    Velocity,

    /**
    The range in which a grain can randomly spawn with a different offset on top of the inital `offset` value.
    This is bipolar, so the random offset can be either smaller or greater than the global `offset` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::OffsetSpread, 0.0); // no random offset added
    g.set_parameter(GranulatorParameter::OffsetSpread, 1.0); // full random offset in range [0, source_length] bipolarly added
    ```
    */
    OffsetSpread,
    /**
    The range in which a grain can randomly spawn with a different grain size on top of the inital `grain_size` value.
    This is bipolar, so the random grain size can be either smaller or greater than the global `grain_size` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::GrainSizeSpread, 0.0); // no random grain size added
    g.set_parameter(GranulatorParameter::GrainSizeSpread, 1.0); // full random grain size in range [1ms, source_length - offset] bipolarly added
    ```
    */
    GrainSizeSpread,
    /**
    The range in which a grain can randomly spawn with a different pitch on top of the inital `pitch` value.
    This is bipolar, so the random pitch can be either smaller or greater than the global `pitch` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::PitchSpread, 0.0); // no random pitch added
    g.set_parameter(GranulatorParameter::PitchSpread, 1.0); // full random pitch in range [0.1, 10.0] bipolarly added
    ```
    */
    PitchSpread,
    /**
    The range in which a grain can randomly spawn with a different velocity on top of the inital `velocity` value.
    This is bipolar, so the random velocity can be either smaller or greater than the global `velocity` value, but never exceeds the bounds.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::VelocitySpread, 0.0); // no random velocity added
    g.set_parameter(GranulatorParameter::VelocitySpread, 1.0); // full random velocity in range [0.0, 1.0] bipolarly added
    ```
    */
    VelocitySpread,
    /**
    The range in which a grain can randomly spawn with a different delay on top of the inital `delay` value.
    This is unipolar, so the random delay can only be greater than the global `delay` value.

    ### Example

    ```
    use granulator::*;

    let mut g = Granulator::new(48_000);
    g.set_parameter(GranulatorParameter::DelaySpread, 0.0); // no random delay added
    g.set_parameter(GranulatorParameter::DelaySpread, 1.0); // random delay in range [0s, 1s] added
    ```
    */
    DelaySpread,
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
            grains: GrainsVector::new(),
            audio_buffer: None,

            master_volume: 1.0 / MAX_GRAINS as f32,
            active_grains: 1,
            offset: 0,
            grain_size_in_samples: 480,
            pitch: 1.0,
            delay: Duration::ZERO,
            velocity: 1.0,

            max_grain_size_in_ms: 1000.0,

            sp_offset: 0.0,
            sp_grain_size: 0.0,
            sp_pitch: 0.0,
            sp_delay: 0.0,
            sp_velocity: 0.0,

            random_offset_value: 0,
            random_grain_size_value: 480,
            random_pitch_value: 1.0,
            random_delay_value: Duration::ZERO,
            random_velocity_value: 1.0,

            current_id_counter: 0,
            fs,

            rng: Rand32::new(random_memory_location as u64),
        }
    }

    // ==========================
    // SETTER WITH BOUND CHECKING
    // ==========================

    /// Sets the internal master volume, which is dependend on `MAX_GRAINS`.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_master_volume(&mut self, volume: f32) {
        if volume <= 0.0 {
            self.master_volume = 0.0;
        }

        if volume > 0.0 && volume < 5.0 {
            self.master_volume = (volume * 2.0) / MAX_GRAINS as f32;
        }

        if volume >= 5.0 {
            self.master_volume = 10.0 / MAX_GRAINS as f32;
        }
    }

    /// Sets the currently active grains.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_active_grains(&mut self, mut num_grains: usize) {
        if num_grains > MAX_GRAINS {
            num_grains = MAX_GRAINS;
        }

        self.active_grains = num_grains;
    }

    /// Sets the offset/playhead on the currently loaded buffer. Depending on other parameters,
    /// every grain can have a different offset around this value.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_offset(&mut self, offset: usize) {
        if self.audio_buffer.is_some() {
            let buffer_length = self.audio_buffer.as_ref().unwrap().length as usize;
            if offset >= buffer_length {
                self.offset = buffer_length;
            } else {
                self.offset = offset;
            }
        }
    }

    /// Sets the main grain size. Depending on other parameters, every grain can have a different
    /// size around this value.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_grain_size(&mut self, grain_size_in_ms: f32) {
        if self.audio_buffer.is_some() {
            let size_in_samples = ((self.fs as f32 / 1000.0) * grain_size_in_ms) as usize;
            let max_length = self.audio_buffer.as_ref().unwrap().length as usize - self.offset;
            if size_in_samples >= max_length {
                self.grain_size_in_samples = max_length;
            } else {
                self.grain_size_in_samples = size_in_samples;
            }
        }
    }

    /// Sets the current pitch. Depending on other parameters, every grain can have a different
    /// pitch around this value.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_pitch(&mut self, pitch: f32) {
        if self.audio_buffer.is_some() {
            if pitch <= 0.1 {
                self.pitch = 0.1;
            }
            if pitch > 0.1 && pitch < 20.0 {
                self.pitch = pitch;
            }
            if pitch >= 20.0 {
                self.pitch = 20.0;
            }
        }
    }

    /// Sets the current velocity. Depending on other parameters, every grain can have a different
    /// velocity around this value.
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_veloctiy(&mut self, velocity: f32) {
        if self.audio_buffer.is_some() {
            if velocity <= 0.0 {
                self.velocity = 0.0;
            }
            if velocity > 0.0 && velocity < 1.0 {
                self.velocity = velocity;
            }
            if velocity >= 1.0 {
                self.velocity = 1.0;
            }
        }
    }

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
            let max_grain_size =
                (self.audio_buffer.as_ref().unwrap().length / self.fs as f32) * 1000.0;
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

    /// Sets a `GranulatorParameter` with bound checking. If the given value is less than 0, it will
    /// be kept at 0. If the the given value is more than 1, it will be kept at 1.
    ///
    ///
    /// This should be updated in a dedicated update task/thread in regular intervals < 20ms.
    pub fn set_parameter(&mut self, parameter: GranulatorParameter, value: f32) {
        if self.audio_buffer.is_some() {
            let mut parameter_value = value;
            if value < 0.0 {
                parameter_value = 0.0;
            }
            if value > 1.0 {
                parameter_value = 1.0;
            }

            match parameter {
                ActiveGrains => {
                    self.active_grains = (parameter_value * MAX_GRAINS as f32) as usize;
                }
                Offset => {
                    self.offset =
                        (parameter_value * self.audio_buffer.as_ref().unwrap().length) as usize;
                }
                GrainSize => {
                    let size_in_ms = parameter_value * 1000.0;
                    let size_in_samples = ((self.fs as f32 / 1000.0) * size_in_ms) as usize;
                    let max_length =
                        self.audio_buffer.as_ref().unwrap().length as usize - self.offset;
                    if size_in_samples >= max_length {
                        self.grain_size_in_samples = max_length;
                    } else {
                        self.grain_size_in_samples = size_in_samples;
                    }
                }
                Pitch => self.pitch = 10.0.powf(parameter_value * 2.0 - 1.0),
                Delay => self.delay = Duration::from_millis((parameter_value * 1000.0) as u64),
                Velocity => self.velocity = parameter_value,
                MasterVolume => self.master_volume = parameter_value,
                OffsetSpread => self.sp_offset = parameter_value,
                GrainSizeSpread => self.sp_grain_size = parameter_value,
                PitchSpread => self.sp_pitch = parameter_value,
                DelaySpread => self.sp_delay = parameter_value,
                VelocitySpread => self.sp_velocity = parameter_value,
            }
        }
    }

    // ==============
    // AUDIO CALLBACK
    // ==============

    /**
    Returns a cummulated sample value of all grains with master volume and soft clipping applied.

    Use this in the audio callback.

    ## Example

    ```
    // some audio callback function
    fn audio_handler(buffer: &mut [f32; 64]) {

        // should be wrapped in Arc<Mutex<_>> or is part of a critical section
        // in the main entry point of the program
        let mut g = granulator::Granulator::new(48_000);

        // lock the granulator object since it has to live on two different threads/tasks
        {
            let mut g_locked = g; // get MutexGuard or apply the closure
            for sample in 0..buffer.len() {
                buffer[sample] = g_locked.get_next_sample();
            }
        }
    }

    ```
    */
    pub fn get_next_sample(&mut self) -> f32 {
        if self.audio_buffer.is_some() {
            soft_clip(self.grains.get_next_sample() * self.master_volume)
        } else {
            0.0
        }
    }

    // ========================
    // AUDIO BUFFER INTERACTION
    // ========================

    /// Sets a new audio buffer for the algorithm to work on.
    pub fn set_audio_buffer(&mut self, buffer: &[f32]) {
        // create slice buffer
        self.audio_buffer = Some(BufferSlice::from_slice(buffer));
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
        for grain in self.grains.get_mut_grains() {
            if grain.finished {
                remove_ids.push(grain.id).unwrap();
            }
        }

        // remove all finished active grains
        for id in remove_ids {
            self.scheduler.remove_grain(id).unwrap();
            self.grains.remove_grain(id).unwrap();
        }
    }

    fn activate_grains(&mut self, ids: &Vec<usize, MAX_GRAINS>) {
        if self.audio_buffer.is_some() {
            for id in ids {
                let velocity = self.get_new_velocity();
                let pitch = self.get_new_pitch();
                let offset = self.get_new_offset();
                let grain_size = self.get_new_grain_size();
                self.grains
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

    fn spawn_future_grains(&mut self) {
        // the difference between all future grains and number of active grains should be spawned, but never less than zero
        let to_be_spawned = self
            .active_grains
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

    // ==============================c
    // PARAMETER RUNTIME CALCULATIONS
    // ==============================

    fn get_new_offset(&mut self) -> usize {
        if self.sp_offset >= SPREAD_ESPILON {
            self.get_spreaded(Offset);
            let mut random_offset = self.random_offset_value;

            let max_length = self.audio_buffer.as_ref().unwrap().length as usize - 1000;

            if random_offset >= max_length {
                random_offset = max_length;
            }

            random_offset
        } else {
            self.offset
        }
    }

    fn get_new_grain_size(&mut self) -> f32 {
        if self.sp_grain_size >= SPREAD_ESPILON {
            self.get_spreaded(GrainSize);

            self.random_grain_size_value as f32
        } else {
            self.grain_size_in_samples as f32
        }
    }

    fn get_new_window(&self) -> WindowFunction {
        WindowFunction::Sine
    }

    fn get_new_source(&self) -> Source {
        Source::AudioFile
    }

    fn get_new_pitch(&mut self) -> f32 {
        if self.sp_pitch >= SPREAD_ESPILON {
            self.get_spreaded(Pitch);
            let mut random_pitch = self.random_pitch_value;

            if random_pitch <= 0.1 {
                random_pitch = 0.1;
            }
            if random_pitch >= 10.0 {
                random_pitch = 10.0;
            }

            random_pitch
        } else {
            self.pitch
        }
    }

    fn get_new_delay(&mut self) -> Duration {
        if self.sp_delay >= SPREAD_ESPILON {
            self.get_spreaded(Delay);

            self.random_delay_value
        } else {
            self.delay
        }
    }

    fn get_new_velocity(&mut self) -> f32 {
        if self.sp_velocity >= SPREAD_ESPILON {
            self.get_spreaded(Velocity);
            let mut random_velocity = self.random_velocity_value;

            if random_velocity < 0.0 {
                random_velocity = 0.0;
            }
            if random_velocity > 1.0 {
                random_velocity = 1.0;
            }

            random_velocity
        } else {
            self.velocity
        }
    }

    fn get_spreaded(&mut self, parameter: GranulatorParameter) {
        match parameter {
            Offset => {
                let range = self.audio_buffer.as_ref().unwrap().length;
                let random_offset =
                    (self.sp_offset * get_random_bipolar_float(&mut self.rng) * range) as isize;

                let signed_offset = self.offset as isize + random_offset;
                self.random_offset_value = signed_offset.clamp(0, range as isize) as usize;
            }
            GrainSize => {
                let range = self.audio_buffer.as_ref().unwrap().length;
                let random_grain_size =
                    (self.sp_grain_size * get_random_bipolar_float(&mut self.rng) * range) as isize;
                let signed_grain_size = self.grain_size_in_samples as isize + random_grain_size;
                self.random_grain_size_value = signed_grain_size.clamp(0, range as isize) as usize;
            }
            Pitch => {
                self.random_pitch_value =
                    self.pitch + self.sp_pitch * get_random_bipolar_float(&mut self.rng) * 5.0;
            }
            Delay => {
                let random_duration_in_ms =
                    self.sp_delay * get_random_unipolar_float(&mut self.rng) * 1000.0;
                self.random_delay_value =
                    self.delay + Duration::from_millis(random_duration_in_ms as u64);
            }
            Velocity => {
                self.random_velocity_value =
                    self.velocity + self.sp_velocity * get_random_bipolar_float(&mut self.rng);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 100];
        m.set_audio_buffer(&buffer);

        let mut ids = Vec::new();

        assert!(m.grains.get_grains().len() == 0);

        for _ in 0..MAX_GRAINS {
            ids.push(m.get_new_id()).unwrap();
        }

        m.activate_grains(&ids);

        assert!(m.grains.get_grains().len() == MAX_GRAINS);
    }

    #[test]
    fn spawn_all_grains() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 100];
        m.set_audio_buffer(&buffer);
        m.set_active_grains(MAX_GRAINS);

        m.spawn_future_grains();

        assert!(m.scheduler.future_vector.len() == MAX_GRAINS);
    }

    #[test]
    fn remove_all_grains() {
        // setup unit test
        let mut m = Granulator::new(FS);
        let buffer = [1_f32; 10000];

        let mut check_slice: Vec<usize, MAX_GRAINS> = Vec::new();
        let mut zero_slice: Vec<usize, MAX_GRAINS> = Vec::new();
        for i in 0..MAX_GRAINS {
            check_slice.push(i).unwrap();
            zero_slice.push(0).unwrap();
        }

        m.set_audio_buffer(&buffer);
        m.set_active_grains(MAX_GRAINS);
        m.set_grain_size(10.0);

        // update scheduler
        m.spawn_future_grains();
        let ids = m.scheduler.update_clock(Duration::from_millis(20));

        assert!(ids == check_slice);

        m.activate_grains(&ids);
        m.remove_finished_grains();

        assert!(m.grains.get_grains().len() == MAX_GRAINS);
        assert!(m.scheduler.future_vector.len() == MAX_GRAINS);

        // finish all grains
        for _ in 0..481 {
            m.get_next_sample();
        }

        // update schedular
        m.spawn_future_grains();
        let ids = m.scheduler.update_clock(Duration::from_millis(20));
        assert!(ids == Vec::<usize, MAX_GRAINS>::new());
        m.activate_grains(&ids);
        m.remove_finished_grains();

        assert!(m.grains.get_grains().len() == 0);
        assert!(m.scheduler.future_vector.len() == 0);

        // next cycle

        m.spawn_future_grains();
        assert!(m.grains.get_grains().len() == 0);
        assert!(m.scheduler.future_vector.len() == MAX_GRAINS);

        let ids = m.scheduler.update_clock(Duration::from_millis(20));
        m.activate_grains(&ids);
        assert!(m.grains.get_grains().len() == MAX_GRAINS);
        assert!(m.scheduler.future_vector.len() == MAX_GRAINS);
    }

    #[test]
    fn set_a_grain_size() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 10000];
        m.set_audio_buffer(&buffer);

        m.set_grain_size(100.0);
        m.set_active_grains(1);

        assert!(m.grain_size_in_samples == 4800);
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
