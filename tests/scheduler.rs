mod index_mapping;

use assert2::*;
use granulator::grain_vector::*;
use granulator::scheduler::Scheduler;
use index_mapping::get_new_index;
use std::time::Duration;

#[test]
fn spawn_grain() {
    let s = Scheduler::new();
    check!(s.activate_grain(get_new_index()).is_ok());
}

#[test]
fn delayed_spawn() {
    let delay = 1000;

    let mut s = Scheduler::new();

    let index = get_new_index();

    // schedule new grain
    check!(s
        .schedule_grain(index, Duration::from_millis(delay))
        .is_ok());

    for _ in 0..delay - 1 {
        s.update_clock();

        check!(get_grain(index).err().expect("Should be the index!") == index);
    }

    s.update_clock();

    let should_be_finished = get_grain(index).unwrap().is_finished();
    check!(should_be_finished);
}
