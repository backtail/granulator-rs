use assert2::*;
use granulator::grain::*;
use granulator::source::*;
use granulator::window_function::*;

pub const ATT_60_DB: f32 = 0.001;
pub const ATT_80_DB: f32 = 0.0001;
pub const ATT_100_DB: f32 = 0.00001;

struct TestGrain {
    grain: Grain,
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

    // last sample in buffer has to be 0 if envelope is finished
    check!(
        test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
        "Last sample is not 0!",
    );
    check!(g.is_finished(), "Envelope function was not finished yet!",);
}
