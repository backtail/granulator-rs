use super::grain::Grain;
use super::scheduler;
use super::source::Source;
use super::window_function::WindowFunction;

pub const MAX_GRAINS: usize = 64;

pub struct Granulator {
    pub grains: [Grain; MAX_GRAINS],
    pub num_active_grains: usize,
    pub fs: usize,
    // parameters
}

impl Granulator {
    pub fn new(fs: usize) -> Self {
        Granulator {
            grains: [Grain::new(fs); MAX_GRAINS],
            num_active_grains: 0,
            fs,
        }
    }

    pub fn update_active_grains(&mut self) {
        for i in 0..self.num_active_grains {
            self.grains[i].update_envelope();
            self.grains[i].update_source_sample();
        }
    }

    pub fn start_first_mode(
        &mut self,
        num_grains: usize,
        grain_size: f32,        // in ms
        grain_size_spread: f32, // in ms
        offset: usize,          // in samples
        window: WindowFunction,
        source: Source,
        source_length: usize,
    ) {
        if num_grains < MAX_GRAINS {
            self.num_active_grains = num_grains;
        } else {
            self.num_active_grains = MAX_GRAINS;
        }
        if num_grains == 0 {
            self.num_active_grains = 1;
        }

        let spread = grain_size_spread / num_grains as f32;

        for i in 0..num_grains {
            scheduler::activate_grain(
                &mut self.grains[i],
                grain_size + i as f32 * spread,
                offset,
                window,
                source,
                source_length,
            )
        }
    }

    pub fn reactivate_grains(&mut self) {
        for i in 0..self.num_active_grains {
            scheduler::reactivate_grains(&mut self.grains[i]);
        }
    }

    pub fn change_offset(&mut self, offset: usize) {
        for i in 0..self.num_active_grains {
            // core::mem::replace(&mut self.grains[i].source_offset, Some(offset)).unwrap();
            self.grains[i].source_offset = Some(offset);
        }
    }
}
