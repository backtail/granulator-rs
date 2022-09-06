use super::grain::Grain;
use super::grain_vector;
use super::manager::MAX_GRAINS;

use core::time::Duration;
use heapless::Vec;

pub struct Future {
    time: Duration,
    id: usize,
}

impl Future {
    fn new(time: Duration, id: usize) -> Self {
        Future { time, id }
    }
}

pub struct Scheduler {
    master_clock_counter: Duration,
    future_vector: Vec<Future, MAX_GRAINS>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            master_clock_counter: Duration::ZERO,
            future_vector: Vec::new(),
        }
    }

    pub fn update_clock(&mut self) {
        self.master_clock_counter += Duration::from_millis(1);

        for grain in &self.future_vector {
            if grain.time <= self.master_clock_counter {
                self.activate_grain(grain.id).ok().unwrap();
            }
        }
    }

    // size in ms between 1ms..100ms
    pub fn activate_grain(&self, id: usize) -> Result<(), Grain> {
        grain_vector::push_grain(id)
    }

    pub fn schedule_grain(&mut self, id: usize, delay: Duration) -> Result<(), Future> {
        self.future_vector
            .push(Future::new(self.master_clock_counter + delay, id))
    }
}
