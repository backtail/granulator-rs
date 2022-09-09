mod util;

use assert2::*;
use granulator::grain_vector::*;
use granulator::scheduler::Scheduler;
use std::time::Duration;
use util::*;

#[test]
fn spawn_grain() {
    let s = Scheduler::new();
    check!(s.activate_grain(get_new_index()).is_ok());

    flush_grains();
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

    check!(get_grain(index).is_ok());

    flush_grains();
}

#[test]
fn automated_removal_of_a_finished_grain() {
    let mut s = Scheduler::new();
    let id = get_new_index();

    // immediatly start grain and update clock to spawn the grain
    check!(s.schedule_grain(id, Duration::ZERO).is_ok());
    s.update_clock();

    // setup buffer size
    const BUFFER_LENGTH: usize = 512;

    // grain size needs to be smaller than buffer length for it to finish in one callback
    check!(get_grain(id).unwrap().get_grain_size_in_samples() < BUFFER_LENGTH);

    // simulate audio callback
    for _ in 0..BUFFER_LENGTH {
        get_next_sample();
    }

    // grain should exist before clock update
    check!(get_grain(id).ok().is_some());

    s.update_clock();

    // grain should be removed after clock update
    check!(get_grain(id).err().is_some());
}
