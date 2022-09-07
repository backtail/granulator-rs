use super::scheduler::Scheduler;

pub const MAX_GRAINS: usize = 64;
pub const FS: usize = 48_000;

pub struct Granulator {
    pub scheduler: Scheduler,
    // parameters
}

impl Granulator {
    pub fn new() -> Self {
        Granulator {
            scheduler: Scheduler::new(),
        }
    }
}
