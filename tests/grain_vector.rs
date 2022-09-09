mod util;

use assert2::*;
use granulator::grain_vector::*;
use util::*;

#[test]
fn push_and_remove_a_grain() {
    let id = get_new_index();

    check!(push_grain(id).is_ok());

    check!(remove_grain(id).is_ok());

    check!(get_grain(id).is_err());
}

#[test]
fn flush_grains_vector() {
    for _ in 0..10 {
        check!(push_grain(get_new_index()).is_ok());
    }
    flush_grains();
}

#[test]
fn remove_grain_from_empty_vector() {
    flush_grains();
    check!(remove_grain(get_new_index()).is_err());
}

#[test]
fn get_a_default_grain() {
    let id = get_new_index();

    check!(push_grain(id).is_ok());

    check!(get_grain(id).is_ok());

    flush_grains();
}

#[test]
fn is_a_grain_finished() {
    let id = get_new_index();

    // setup audio callback length
    check!(push_grain(id).is_ok());
    const BUFFER_LENGTH: usize = 512;

    // grain size needs to be smaller than buffer length for it to finish in one callback
    check!(get_grain(id).unwrap().get_grain_size_in_samples() < BUFFER_LENGTH);

    // simulate audio callback
    for _ in 0..BUFFER_LENGTH {
        update_envolopes();
    }

    // grain should be finished
    check!(get_grain(id).unwrap().finished);

    flush_grains();
}
