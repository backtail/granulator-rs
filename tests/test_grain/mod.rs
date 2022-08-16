use granulator::{grain::Grain, source::Source, window_function::WindowFunction};

pub struct TestGrain {
    pub grain: Grain,
}

impl TestGrain {
    pub fn new(fs: usize) -> Self {
        TestGrain {
            grain: Grain::new(fs),
        }
    }

    pub fn activate_grain_with_no_source(&mut self, grain_size_in_ms: f32, window: WindowFunction) {
        let offset = 0; // dummy value
        let source = Source::Synthetic; // dummy value
        let source_length = 0; // dummy value

        self.grain
            .activate(grain_size_in_ms, offset, window, source, source_length)
    }
}

pub fn check_bounds_setup(window: WindowFunction) -> (Grain, usize) {
    let fs: usize = 48_000;

    // use a not "beautiful" to take floating point errors in consideration
    let grain_size_in_ms: f32 = 0.9;

    // include zero in front an at the end
    let test_buffer_length: usize = ((grain_size_in_ms * fs as f32) / 1000.0) as usize + 2;

    // create test grain
    let mut t = TestGrain::new(fs);

    // initialize grain only for envelope testing
    t.activate_grain_with_no_source(grain_size_in_ms, window);

    // return actual grain
    let g = t.grain;

    (g, test_buffer_length)
}
