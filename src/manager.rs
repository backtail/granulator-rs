// data management
use heapless::Vec;

// scheduler specific
use super::scheduler::Scheduler;
use core::time::Duration;

// grain vector
use crate::grains_vector::GrainsVector;

// audio processing
use super::audio_tools::soft_clip;

// global constants
pub const MAX_GRAINS: usize = 64;
pub const FS: usize = 48_000;

pub struct Granulator {
    scheduler: Scheduler,
    grains: GrainsVector,
    buffer_pointer: *const f32, // points to the beginning of the buffer

    // parameters
    master_volume: f32,
    active_grains: usize,
    offset: usize,
    grain_size_in_samples: usize,

    sample_length: Option<usize>,

    current_id_counter: usize,
}

impl Granulator {
    pub fn new(buffer_pointer: *const f32) -> Self {
        Granulator {
            scheduler: Scheduler::new(),
            grains: GrainsVector::new(),
            buffer_pointer,

            master_volume: 1.0 / MAX_GRAINS as f32,
            active_grains: 1,
            offset: 0,
            grain_size_in_samples: 0,

            sample_length: None,

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
        if self.sample_length.is_some() {
            if offset >= self.sample_length.unwrap() {
                self.offset = self.sample_length.unwrap();
            } else {
                self.offset = offset;
            }
        }
    }

    pub fn set_grain_size(&mut self, grain_size_in_ms: f32) {
        if self.sample_length.is_some() {
            let size_in_samples = (FS as f32 / (grain_size_in_ms * 1000.0)) as usize;
            let max_length = self.sample_length.unwrap() - self.offset;
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
        let mut out_sample = 0_f32;

        for grain in self.grains.get_mut_grains() {
            out_sample += grain.get_next_sample();
        }

        soft_clip(out_sample * self.master_volume)
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
        for id in &remove_ids {
            self.grains.remove_grain(*id).unwrap();
        }

        // after removing, the grains vector is the smallest and can potentially spawn/activate the most grains
        self.activate_grains(&activate_these_ids);

        // the difference between the current grain vector and number of active grains should be spawned
        let to_be_spawned = self.active_grains - self.grains.get_grains().len();

        // spawn future grains
        for _ in 0..to_be_spawned {
            let id = self.get_new_id();
            let delay = self.get_new_delay();
            self.scheduler.schedule_grain(id, delay).unwrap();
        }
    }

    fn activate_grains(&mut self, ids: &Vec<usize, MAX_GRAINS>) {
        for id in ids {
            self.activate_grain(id).unwrap();
        }
    }

    fn activate_grain(&mut self, id: &usize) -> Result<(), usize> {
        let offset = self.get_new_offset();
        let size = self.get_new_grain_size();

        let sub_slice =
            unsafe { core::ptr::slice_from_raw_parts(self.buffer_pointer.add(offset), size) };
        self.grains.push_grain(*id, sub_slice)
    }

    fn get_new_id(&mut self) -> usize {
        if self.current_id_counter >= usize::MAX - 1 {
            self.current_id_counter = 0;
        }
        self.current_id_counter += 1;

        self.current_id_counter
    }

    // ==============================
    // PARAMETER RUNTIME CALCULATIONS
    // ==============================

    fn get_new_offset(&self) -> usize {
        self.offset
    }

    fn get_new_grain_size(&self) -> usize {
        self.grain_size_in_samples
    }

    fn get_new_delay(&self) -> Duration {
        core::time::Duration::ZERO
    }
}
