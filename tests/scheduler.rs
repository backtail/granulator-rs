use assert2::*;
use granulator::{grain::Grain, scheduler, source, window_function};

#[test]
fn spawn_grain() {
    let mut grain = Grain::new(48000);

    // check if grain has not been activated yet
    check!(grain.is_finished());

    scheduler::activate_grain(
        &mut grain,
        2.0,
        0,
        window_function::WindowFunction::Sine,
        source::Source::Synthetic,
        0,
    );

    // check if grain has started
    check!(!grain.is_finished());
}
