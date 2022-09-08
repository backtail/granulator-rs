use super::grain::Grain;
use super::grain_vector;
use super::manager::MAX_GRAINS;

use core::time::Duration;
use heapless::Vec;

#[derive(Debug)]
pub struct TimeInfo {
    id: usize,
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
    future_vector: Vec<TimeInfo, MAX_GRAINS>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            master_clock_counter: Duration::ZERO,
            future_vector: Vec::new(),
        }
    }

    pub fn update_clock(&mut self) {
        // increase counter by 1
        self.master_clock_counter += Duration::from_millis(1);

        // remove all finished grains
        self.future_vector.retain(|future_grain| {
            let real_grain = grain_vector::get_grain(future_grain.id);

            let mut is_finished = false;

            if real_grain.is_ok() {
                is_finished = real_grain.unwrap().finished;
                if is_finished {
                    if grain_vector::remove_grain(future_grain.id).is_err() {
                        panic!("Didn't remove the grain!")
                    }
                }
            }

            !is_finished
        });

        // check if grains crossed the start time
        for future_grain in &self.future_vector {
            if future_grain.start <= self.master_clock_counter {
                self.activate_grain(future_grain.id).ok().unwrap();
            }
        }
    }

    pub fn activate_grain(&self, id: usize) -> Result<(), Grain> {
        grain_vector::push_grain(id)
    }

    pub fn schedule_grain(&mut self, id: usize, delay: Duration) -> Result<(), TimeInfo> {
        self.future_vector
            .push(TimeInfo::new(id, self.master_clock_counter + delay))
    }
}
