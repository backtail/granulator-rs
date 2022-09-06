mod index_mapping;

use assert2::*;
use granulator::grain_vector::*;
use index_mapping::get_new_index;

#[test]
fn push_and_remove_a_grain() {
    let id = get_new_index();

    check!(push_grain(id).is_ok());

    check!(remove_grain(id).is_ok());
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
fn get_a_grain() {
    let id = get_new_index();

    check!(push_grain(id).is_ok());

    check!(get_grain(id).is_ok());

    flush_grains();
}
