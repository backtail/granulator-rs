use assert2::*;
use granulator::manager::{Granulator, GRAINS};
use std::time::Duration;

#[test]
fn spawn_grain() {
    let m = Granulator::new();

    // lock
    {
        // check if grain has not been activated yet
        check!(GRAINS.lock().get_mut(1).unwrap().is_finished());
    }
    m.scheduler.activate_grain(1);

    // lock
    {
        // check if grain has started
        check!(!GRAINS.lock().get_mut(1).unwrap().is_finished());
    }
}

#[test]
fn delayed_spawn() {
    let mut m = Granulator::new();

    // activate grain with 10ms delay
    m.scheduler.schedule_grain(0, Duration::from_millis(10));

    for i in 0..9 {
        m.scheduler.update_clock();

        // check if grain has not been activated yet
        check!(
            GRAINS.lock().get_mut(0).unwrap().is_finished(),
            "At time {}ms",
            i
        );
    }

    m.scheduler.update_clock();

    // lock
    {
        // check if grain has been activated
        check!(
            !GRAINS.lock().get_mut(0).unwrap().is_finished(),
            "Grain has not been activated after delay!"
        );
    }
}
