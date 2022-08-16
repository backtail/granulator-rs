use assert2::*;
use granulator::window_function::WindowFunction;

mod test_grain;

#[test]
fn check_bounds_envelope_sine() {
    // setup grain
    let (mut g, test_buffer_length) = test_grain::check_bounds_setup(WindowFunction::Sine);

    // create test buffer
    let mut test_buffer: Vec<f32> = vec![];

    // first sample is by definition 0
    check!(
        g.envelope_value == 0.0,
        "First sample was not 0.0, but {}!",
        g.envelope_value
    );

    // run through the grain until finished and store its values for further investigation
    for _i in 0..test_buffer_length {
        g.update_envelope();
        test_buffer.push(g.envelope_value);
    }

    for sample in &test_buffer {
        // check upper bound
        check!(*sample <= 1.0, "Envelope grew bigger than 1!");

        // check lower bound
        check!(*sample >= 0.0, "Envelope got negative!");
    }

    // last sample in buffer has to be 0 if envelope is finished
    check!(
        test_buffer[test_buffer_length - 1] == 0.0,
        "Last sample is not 0!",
    );
    check!(g.is_finished(), "Envelope function was not finished yet!",);
}

#[test]
fn check_bounds_envelope_hamming() {
    // setup grain
    let (mut g, test_buffer_length) = test_grain::check_bounds_setup(WindowFunction::Hamming);

    // create test buffer
    let mut test_buffer: Vec<f32> = vec![];

    // first sample is by definition 0
    check!(
        g.envelope_value == 0.0,
        "First sample was not 0.0, but {}!",
        g.envelope_value
    );

    // run through the grain until finished and store its values for further investigation
    for _i in 0..test_buffer_length {
        g.update_envelope();
        test_buffer.push(g.envelope_value);
    }

    for sample in &test_buffer {
        // check upper bound
        check!(*sample <= 1.0, "Envelope grew bigger than 1!");

        // check lower bound
        check!(*sample >= 0.0, "Envelope got negative!");
    }

    // last sample in buffer has to be 0 if envelope is finished
    check!(
        test_buffer[test_buffer_length - 1] == 0.0,
        "Last sample is not 0!",
    );
    check!(g.is_finished(), "Envelope function was not finished yet!",);
}

#[test]
fn check_bounds_envelope_hann() {
    // setup grain
    let (mut g, test_buffer_length) = test_grain::check_bounds_setup(WindowFunction::Hann);

    // create test buffer
    let mut test_buffer: Vec<f32> = vec![];

    // first sample is by definition 0
    check!(
        g.envelope_value == 0.0,
        "First sample was not 0.0, but {}!",
        g.envelope_value
    );

    // run through the grain until finished and store its values for further investigation
    for _i in 0..test_buffer_length {
        g.update_envelope();
        test_buffer.push(g.envelope_value);
    }

    for sample in &test_buffer {
        // check upper bound
        check!(*sample <= 1.0, "Envelope grew bigger than 1!");

        // check lower bound
        check!(*sample >= 0.0, "Envelope got negative!");
    }

    // last sample in buffer has to be 0 if envelope is finished
    check!(
        test_buffer[test_buffer_length - 1] == 0.0,
        "Last sample is not 0!",
    );
    check!(g.is_finished(), "Envelope function was not finished yet!",);
}
