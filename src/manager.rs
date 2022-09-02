use spin::Mutex;

use super::grain::Grain;
use super::grain::DEFAULT_GRAIN;
use super::scheduler::Scheduler;

pub const MAX_GRAINS: usize = 64;
pub const FS: usize = 48_000;
pub static GRAINS: Mutex<[Grain; MAX_GRAINS]> = Mutex::new([DEFAULT_GRAIN; MAX_GRAINS]);

pub struct Granulator {
    pub scheduler: Scheduler, // parameters
}

impl Granulator {
    pub fn new() -> Self {
        reset();
        for i in 0..MAX_GRAINS {
            GRAINS.lock().get_mut(i).unwrap().id = i;
        }

        Granulator {
            scheduler: Scheduler::new(),
        }
    }
}

fn reset() {
    for i in 0..MAX_GRAINS {
        GRAINS.lock().get_mut(i).unwrap().reset();
    }
}
