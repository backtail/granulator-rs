use super::source::Source;
use super::window_function::WindowFunction;

#[derive(Clone, Copy)]
pub struct Grain {
    // envelope variables
    window: Option<WindowFunction>,
    window_parameter: Option<f32>,
    envelope_position: u32,  // in samples between 0..grain_size
    pub envelope_value: f32, // between 0..1

    // source variables
    source: Option<Source>,
    source_length: Option<usize>,     // in samples
    pub source_offset: Option<usize>, // in samples
    relative_position: usize,         // between 0..grain_size
    pub source_position: usize,       // between source_offset..source_length

    // grain variables
    grain_size: Option<f32>, // in samples
    finished: bool,

    // misc
    fs: usize,
}

impl Grain {
    pub fn new(fs: usize) -> Self {
        Grain {
            window: None,
            window_parameter: None,
            envelope_position: 0,
            envelope_value: 0.0,

            source: None,
            source_length: None,
            source_offset: None,
            relative_position: 0,
            source_position: 0,

            grain_size: None,
            finished: true,

            fs,
        }
    }

    pub fn activate(
        &mut self,
        grain_size: f32,
        offset: usize,
        window: WindowFunction,
        source: Source,
        source_length: usize,
    ) {
        // setting up envelope
        self.window = Some(window);

        // setting up source
        self.source = Some(source);
        self.source_offset = Some(offset);
        self.source_length = Some(source_length);

        // setting up grain
        let size = (self.fs as f32 * grain_size) / 1000.0; // convert ms into samples

        self.grain_size = Some(size);
        self.finished = false; // start grain
    }

    pub fn is_finished(&mut self) -> bool {
        self.finished
    }

    pub fn reactivate(&mut self) {
        self.envelope_position = 0;
        self.envelope_value = 0.0;
        self.relative_position = 0;
        self.source_position = self.source_offset.unwrap();

        self.finished = false;
    }

    pub fn update_envelope(&mut self) {
        if !self.finished {
            let current_position = self.envelope_position as f32;
            self.envelope_value = self.window.as_ref().unwrap().get_envelope_value(
                current_position,
                self.grain_size.unwrap(),
                self.window_parameter,
            );
            if current_position < self.grain_size.unwrap() {
                self.envelope_position += 1;
            } else {
                self.finished = true;
                self.envelope_value = 0.0;
            }
        }
    }

    // TODO
    // many parameters may be calculated before creating the grain

    // if offset + size > length { doesn't fitclear -> find new position }
    pub fn update_source_sample(&mut self) {
        if !self.finished {
            self.relative_position += 4;
            self.source_position = self
                .source
                .as_ref()
                .unwrap()
                .get_source_sample(self.relative_position, self.source_offset.unwrap());
            if self.source_position
                > self.source_length.unwrap() - self.grain_size.unwrap() as usize
            {
                self.source_position = self.source_offset.unwrap();
                self.finished = true;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use assert2::*;

    struct TestGrain {
        grain: Grain,
    }

    pub const ATT_60_DB: f32 = 0.001;

    impl TestGrain {
        pub fn new(fs: usize) -> Self {
            TestGrain {
                grain: Grain::new(fs),
            }
        }

        pub fn activate_grain_with_no_source(
            &mut self,
            grain_size_in_ms: f32,
            window: WindowFunction,
        ) {
            let offset = 0; // dummy value
            let source = Source::Synthetic; // dummy value
            let source_length = 0; // dummy value

            self.grain
                .activate(grain_size_in_ms, offset, window, source, source_length)
        }
    }

    #[test]
    fn check_bounds_envelope_sine() {
        const FS: usize = 48_000;

        // use a not "beautiful" to take floating point errors in consideration
        const GRAIN_SIZE_IN_MS: f32 = 0.9;

        // include zero in front an at the end
        const TEST_BUFFER_LENGTH: usize = ((GRAIN_SIZE_IN_MS * FS as f32) / 1000.0) as usize + 2;

        // setup grain
        let mut t = TestGrain::new(FS);

        // create test buffer
        let mut test_buffer = [0.0_f32; TEST_BUFFER_LENGTH];

        t.activate_grain_with_no_source(GRAIN_SIZE_IN_MS, WindowFunction::Sine);

        // transform TestGrain into real grain
        let mut g = t.grain;

        // first sample is by definition 0
        check!(
            g.envelope_value == 0.0,
            "First sample was not 0.0, but {}!",
            g.envelope_value
        );

        // run through the grain until finished and store its values for further investigation
        for i in 0..TEST_BUFFER_LENGTH {
            g.update_envelope();
            test_buffer[i] = g.envelope_value;
        }

        for i in 0..TEST_BUFFER_LENGTH {
            // check upper bound
            check!(
                test_buffer[i] <= 1.0,
                "Envelope grew bigger than 1 at position {}",
                i
            );

            // check lower bound
            check!(
                test_buffer[i] >= 0.0,
                "Envelope got negative at position {}",
                i
            );
        }

        // print sine envelope buffer if true
        check!(true, "{:?}", test_buffer);

        // last sample in buffer has to be 0 if envelope is finished
        check!(
            test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
            "Last sample is not 0!",
        );
        check!(g.is_finished(), "Envelope function was not finished yet!",);
    }

    #[test]
    fn check_bounds_envelope_hamming() {
        const FS: usize = 48_000;

        // use a not "beautiful" to take floating point errors in consideration
        const GRAIN_SIZE_IN_MS: f32 = 0.9;

        // include zero in front an at the end
        const TEST_BUFFER_LENGTH: usize = ((GRAIN_SIZE_IN_MS * FS as f32) / 1000.0) as usize + 2;

        // setup grain
        let mut t = TestGrain::new(FS);

        // create test buffer
        let mut test_buffer = [0.0_f32; TEST_BUFFER_LENGTH];

        t.activate_grain_with_no_source(GRAIN_SIZE_IN_MS, WindowFunction::Hamming);

        // transform TestGrain into real grain
        let mut g = t.grain;

        // first sample is by definition 0
        check!(
            g.envelope_value == 0.0,
            "First sample was not 0.0, but {}!",
            g.envelope_value
        );

        // run through the grain until finished and store its values for further investigation
        for i in 0..TEST_BUFFER_LENGTH {
            g.update_envelope();
            test_buffer[i] = g.envelope_value;
        }

        for i in 0..TEST_BUFFER_LENGTH {
            // check upper bound
            check!(
                test_buffer[i] <= 1.0,
                "Envelope grew bigger than 1 at position {}",
                i
            );

            // check lower bound
            check!(
                test_buffer[i] >= 0.0,
                "Envelope got negative at position {}",
                i
            );
        }

        // print sine envelope buffer if true
        check!(true, "{:?}", test_buffer);

        // last sample in buffer has to be 0 if envelope is finished
        check!(
            test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
            "Last sample is not 0!",
        );
        check!(g.is_finished(), "Envelope function was not finished yet!",);
    }

    #[test]
    fn check_bounds_envelope_hann() {
        const FS: usize = 48_000;

        // use a not "beautiful" to take floating point errors in consideration
        const GRAIN_SIZE_IN_MS: f32 = 0.9;

        // include zero in front an at the end
        const TEST_BUFFER_LENGTH: usize = ((GRAIN_SIZE_IN_MS * FS as f32) / 1000.0) as usize + 2;

        // setup grain
        let mut t = TestGrain::new(FS);

        // create test buffer
        let mut test_buffer = [0.0_f32; TEST_BUFFER_LENGTH];

        t.activate_grain_with_no_source(GRAIN_SIZE_IN_MS, WindowFunction::Hann);

        // transform TestGrain into real grain
        let mut g = t.grain;

        // first sample is by definition 0
        check!(
            g.envelope_value == 0.0,
            "First sample was not 0.0, but {}!",
            g.envelope_value
        );

        // run through the grain until finished and store its values for further investigation
        for i in 0..TEST_BUFFER_LENGTH {
            g.update_envelope();
            test_buffer[i] = g.envelope_value;
        }

        for i in 0..TEST_BUFFER_LENGTH {
            // check upper bound
            check!(
                test_buffer[i] <= 1.0,
                "Envelope grew bigger than 1 at position {}",
                i
            );

            // check lower bound
            check!(
                test_buffer[i] >= 0.0,
                "Envelope got negative at position {}",
                i
            );
        }

        // print sine envelope buffer if true
        check!(true, "{:?}", test_buffer);

        // last sample in buffer has to be 0 if envelope is finished
        check!(
            test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
            "Last sample is not 0!",
        );
        check!(g.is_finished(), "Envelope function was not finished yet!",);
    }
}
