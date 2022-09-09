use super::scheduler::Scheduler;

pub const MAX_GRAINS: usize = 64;
pub const FS: usize = 48_000;

pub struct Granulator {
    pub scheduler: Scheduler,

    // parameters
    pub master_volume: f32,
}

impl Granulator {
    pub fn new() -> Self {
        Granulator {
            scheduler: Scheduler::new(),

            master_volume: 1.0 / MAX_GRAINS as f32,
        }
    }

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
}
