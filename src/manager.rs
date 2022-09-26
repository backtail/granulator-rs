// data management
use heapless::Vec;

// scheduler specific
use super::scheduler::Scheduler;
use core::time::Duration;

// grain vector
use crate::{
    grains_vector::GrainsVector, pointer_wrapper::BufferSlice, source::Source,
    window_function::WindowFunction,
};

// audio processing
use super::audio_tools::soft_clip;

/// The most grains grains that can possibly play at the same time.
///
/// Increasing this will increase processing demands. Only changeable if one recompiles the
/// crate with a different number. This will change in the future.
pub const MAX_GRAINS: usize = 50;

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

    // misc
    current_id_counter: usize,
    fs: usize,
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
        Granulator {
            scheduler: Scheduler::new(),
            grains: GrainsVector::new(),
            audio_buffer: None,

            master_volume: 1.0 / MAX_GRAINS as f32,
            active_grains: 1,
            offset: 0,
            grain_size_in_samples: 480,
            pitch: 1.0,

            current_id_counter: 0,
            fs,
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
                self.grains
                    .push_grain(
                        *id,
                        self.audio_buffer
                            .as_ref()
                            .unwrap()
                            .get_sub_slice(self.get_new_offset(), self.get_new_grain_size()),
                        self.get_new_window(),
                        self.get_new_source(),
                        self.get_new_pitch(),
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

    fn get_new_offset(&self) -> usize {
        self.offset
    }

    fn get_new_grain_size(&self) -> f32 {
        self.grain_size_in_samples as f32
    }

    fn get_new_delay(&self) -> Duration {
        core::time::Duration::ZERO
    }

    fn get_new_window(&self) -> WindowFunction {
        WindowFunction::Sine
    }

    fn get_new_source(&self) -> Source {
        Source::AudioFile
    }

    fn get_new_pitch(&self) -> f32 {
        self.pitch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;

    const FS: usize = 48_000;

    #[test]
    fn get_a_new_index() {
        let mut m = Granulator::new(FS);
        check!(m.current_id_counter == 0);

        for i in 0..MAX_GRAINS {
            let new_id = m.get_new_id();

            check!(new_id == i);
        }
    }

    #[test]
    fn activate_all_grains() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 100];
        m.set_audio_buffer(&buffer);

        let mut ids = Vec::new();

        check!(m.grains.get_grains().len() == 0);

        for _ in 0..MAX_GRAINS {
            ids.push(m.get_new_id()).unwrap();
        }

        m.activate_grains(&ids);

        check!(m.grains.get_grains().len() == MAX_GRAINS);
    }

    #[test]
    fn spawn_all_grains() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 100];
        m.set_audio_buffer(&buffer);
        m.set_active_grains(MAX_GRAINS);

        m.spawn_future_grains();

        check!(m.scheduler.future_vector.len() == MAX_GRAINS);
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

        check!(ids == check_slice);

        m.activate_grains(&ids);
        m.remove_finished_grains();

        check!(m.grains.get_grains().len() == MAX_GRAINS);
        check!(m.scheduler.future_vector.len() == MAX_GRAINS);

        // finish all grains
        for _ in 0..481 {
            m.get_next_sample();
        }

        // update schedular
        m.spawn_future_grains();
        let ids = m.scheduler.update_clock(Duration::from_millis(20));
        check!(ids == Vec::<usize, MAX_GRAINS>::new());
        m.activate_grains(&ids);
        m.remove_finished_grains();

        check!(m.grains.get_grains().len() == 0);
        check!(m.scheduler.future_vector.len() == 0);

        // next cycle

        m.spawn_future_grains();
        check!(m.grains.get_grains().len() == 0);
        check!(m.scheduler.future_vector.len() == MAX_GRAINS);

        let ids = m.scheduler.update_clock(Duration::from_millis(20));
        m.activate_grains(&ids);
        check!(m.grains.get_grains().len() == MAX_GRAINS);
        check!(m.scheduler.future_vector.len() == MAX_GRAINS);
    }

    #[test]
    fn set_a_grain_size() {
        let mut m = Granulator::new(FS);
        let buffer = [0_f32; 10000];
        m.set_audio_buffer(&buffer);

        m.set_grain_size(100.0);
        m.set_active_grains(1);

        check!(m.grain_size_in_samples == 4800);
    }

    #[test]
    fn set_a_sample_rate() {
        let mut m = Granulator::new(FS);

        let result = m.set_sample_rate(1_000);
        check!(m.fs == 48_000);
        check!(result.is_err());

        let result = m.set_sample_rate(300_000);
        check!(m.fs == 48_000);
        check!(result.is_err());

        let result = m.set_sample_rate(44_100);
        check!(m.fs == 44_100);
        check!(result.is_ok());
    }
}
