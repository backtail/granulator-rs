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

// check if grains are being pushed onto vector until it is full
#[test]
fn overflow_grain_vector() {
    for _ in 0..MAX_GRAINS {
        check!(push_grain(get_new_index()).is_ok());
    }
    check!(push_grain(get_new_index()).is_err());
}

#[test]
fn remove_grain_from_empty_vector() {
    check!(remove_grain(get_new_index()).is_err());
}
