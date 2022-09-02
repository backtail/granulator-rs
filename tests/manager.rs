use assert2::*;
use granulator::manager::{Granulator, GRAINS, MAX_GRAINS};

#[test]
fn test_ids() {
    Granulator::new();

    for i in 0..MAX_GRAINS {
        check!(GRAINS.lock().get(i).unwrap().id == i);
    }
}
