use super::grain::Grain;
use super::source::Source;
use super::window_function::WindowFunction;

// size in ms between 1ms..100ms
pub fn activate_grain(
    grain: &mut Grain,
    size: f32,
    offset: usize,
    window: WindowFunction,
    source: Source,
    source_length: usize,
) {
    grain.activate(size, offset, window, source, source_length);
}

pub fn reactivate_grains(grain: &mut Grain) {
    if grain.is_finished() {
        grain.reactivate();
    }
}
