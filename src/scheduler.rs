use super::manager::MAX_GRAINS;

use core::time::Duration;
use heapless::Vec;

#[derive(Debug)]
pub struct TimeInfo {
    pub id: usize,
    start: Duration,
}

impl TimeInfo {
    fn new(id: usize, start: Duration) -> Self {
        TimeInfo { id, start }
    }
}

#[derive(Debug)]
pub struct Scheduler {
    pub master_clock_counter: Duration,
    pub future_vector: Vec<TimeInfo, MAX_GRAINS>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            master_clock_counter: Duration::ZERO,
            future_vector: Vec::new(),
        }
    }

    pub fn update_clock(&mut self, time_step: Duration) -> Vec<usize, MAX_GRAINS> {
        // increase counter by timestep
        self.master_clock_counter += time_step;

        let mut return_vec = Vec::new();

        // check if grains crossed the start time
        for future_grain in &self.future_vector {
            if future_grain.start <= self.master_clock_counter {
                return_vec.push(future_grain.id).unwrap();
            }
        }

        return_vec
    }

    pub fn schedule_grain(&mut self, id: usize, delay: Duration) -> Result<(), TimeInfo> {
        self.future_vector
            .push(TimeInfo::new(id, self.master_clock_counter + delay))
    }
}
