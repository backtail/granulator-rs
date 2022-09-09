mod util;

use assert2::*;
use core::time::Duration;
use granulator::{audio_tools::*, grain_vector::*, manager::*};
use util::*;

#[test]
fn check_master_volume_bounds() {
    let mut m = Granulator::new();

    // immediatly start all possible grains and update clock to spawn them
    for _ in 0..MAX_GRAINS {
        m.scheduler
            .schedule_grain(get_new_index(), Duration::ZERO)
            .unwrap();
    }
    m.scheduler.update_clock();

    // set master volume to max
    m.set_master_volume(1.0);

    // setup buffer size
    const BUFFER_LENGTH: usize = 512;

    // simulate audio callback
    for _ in 0..BUFFER_LENGTH {
        let grains_sample = get_next_sample();
        let output_sample = soft_clip(grains_sample * m.master_volume);

        // let output_sample = grains_sample * m.master_volume;

        check!(output_sample <= 1.0, "Sample is greather than 1.0!");
        check!(output_sample >= -1.0, "Sample is smaller than -1.0!");
    }

    flush_grains();
}
