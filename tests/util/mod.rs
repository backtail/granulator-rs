use granulator::{grain_vector::*, source::Source, window_function::WindowFunction};
use spin::Mutex;

static INDEX: Mutex<usize> = Mutex::new(0);

pub fn get_new_index() -> usize {
    let mut locked_index = INDEX.lock();
    let return_index = *locked_index;
    *locked_index += 1;

    return_index
}

pub fn setup_grain_only_with_window_funtion(id: usize, grain_size_in_ms: f32) -> Result<(), usize> {
    set_grain_parameters(
        id,
        grain_size_in_ms,
        WindowFunction::Sine,
        None,
        Source::Synthetic,
        None,
        None,
    )
}
