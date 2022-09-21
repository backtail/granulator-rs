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

// global constants
pub const MAX_GRAINS: usize = 64;
pub const FS: usize = 48_000;

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

    // misc
    current_id_counter: usize,
}

impl Granulator {
    pub fn new() -> Self {
        Granulator {
            scheduler: Scheduler::new(),
            grains: GrainsVector::new(),
            audio_buffer: None,

            master_volume: 1.0 / MAX_GRAINS as f32,
            active_grains: 1,
            offset: 0,
            grain_size_in_samples: 0,

            current_id_counter: 0,
        }
    }

    // ==========================
    // SETTER WITH BOUND CHECKING
    // ==========================

    pub fn set_master_volume(&mut self, volume: f32) {
        if volume <= 0.0 {
            self.master_volume = 0.0;
        }

        if volume > 0.0 && volume < 1.0 {
            self.master_volume = (volume * 2.0) / MAX_GRAINS as f32;
        }

        if volume >= 1.0 {
            self.master_volume = 2.0 / MAX_GRAINS as f32;
        }
    }

    pub fn set_active_grains(&mut self, mut num_grains: usize) {
        if num_grains > MAX_GRAINS {
            num_grains = MAX_GRAINS;
        }

        self.active_grains = num_grains;
    }

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

    pub fn set_grain_size(&mut self, grain_size_in_ms: f32) {
        if self.audio_buffer.is_some() {
            let size_in_samples = ((FS as f32 / 1000.0) * grain_size_in_ms) as usize;
            let max_length = self.audio_buffer.as_ref().unwrap().length as usize - self.offset;
            if size_in_samples >= max_length {
                self.grain_size_in_samples = max_length;
            } else {
                self.grain_size_in_samples = size_in_samples;
            }
        }
    }

    // ==============
    // AUDIO CALLBACK
    // ==============

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

    pub fn set_audio_buffer(&mut self, buffer: &[f32]) {
        // remove all references to old audio buffer
        self.grains.flush();

        // create slice buffer
        self.audio_buffer = Some(BufferSlice::from_slice(buffer));
    }

    // ====================
    // SCHEDULER MANAGEMENT
    // ====================

    pub fn update_scheduler(&mut self, increase: Duration) {
        // collect to be acticated grains when scheduler clock advances
        let activate_these_ids = self.scheduler.update_clock(increase);

        // collect all finished grain ids
        let mut remove_ids: Vec<usize, MAX_GRAINS> = Vec::new();
        for grain in self.grains.get_mut_grains() {
            if grain.finished {
                remove_ids.push(grain.id).unwrap();
            }
        }

        // remove all finished active grains
        for id in remove_ids {
            self.grains.remove_grain(id).unwrap();
        }

        // after removing, the grains vector is the smallest and can potentially spawn/activate the most grains
        self.activate_grains(activate_these_ids);

        // the difference between the current grain vector and number of active grains should be spawned
        let to_be_spawned = self.active_grains - self.grains.get_grains().len();

        // spawn future grains
        for _ in 0..to_be_spawned {
            let id = self.get_new_id();
            let delay = self.get_new_delay();
            self.scheduler.schedule_grain(id, delay).unwrap();
        }
    }

    fn activate_grains(&mut self, ids: Vec<usize, MAX_GRAINS>) {
        if self.audio_buffer.is_some() {
            for id in ids {
                self.activate_grain(id).unwrap();
            }
        }
    }

    fn activate_grain(&mut self, id: usize) -> Result<(), usize> {
        if self.audio_buffer.is_some() {
            self.grains.push_grain(
                id,
                self.audio_buffer
                    .as_ref()
                    .unwrap()
                    .get_sub_slice(self.get_new_offset(), self.get_new_grain_size()),
                self.get_new_window(),
                self.get_new_source(),
            )
        } else {
            Err(id)
        }
    }

    fn get_new_id(&mut self) -> usize {
        if self.current_id_counter >= usize::MAX - 1 {
            self.current_id_counter = 0;
        }
        self.current_id_counter += 1;

        self.current_id_counter
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
}
