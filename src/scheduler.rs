use super::grain::Grain;
use super::manager::{GRAINS, MAX_GRAINS};

use core::time::Duration;

pub struct Scheduler {
    master_clock_counter: Duration,
    future_table: [Duration; MAX_GRAINS],
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            master_clock_counter: Duration::ZERO,
            future_table: [Duration::MAX; MAX_GRAINS],
        }
    }

    pub fn update_clock(&mut self) {
        self.master_clock_counter = self
            .master_clock_counter
            .checked_add(Duration::from_millis(1))
            .unwrap();
        for i in 0..MAX_GRAINS {
            if self.future_table[i] <= self.master_clock_counter {
                self.activate_grain(i);
            }
        }
    }

    // size in ms between 1ms..100ms
    pub fn activate_grain(&self, id: usize) {
        GRAINS.lock().get_mut(id).unwrap().activate();
    }

    pub fn reactivate_grains(&self, grain: &mut Grain) {
        if grain.is_finished() {
            grain.reactivate();
        }
    }

    pub fn schedule_grain(&mut self, id: usize, delay: Duration) {
        self.future_table[id] = self.master_clock_counter + delay;
    }
}
