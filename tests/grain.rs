// use assert2::*;
// use granulator::{grain::Grain, manager::FS, window_function::WindowFunction};

// const TEST_GRAIN_SIZE_IN_MS: f32 = 0.9;
// const TEST_GRAIN_SIZE_IN_SAMPLES: f32 = (TEST_GRAIN_SIZE_IN_MS * FS as f32) / 1000.0;

// const TEST_BUFFER_LENGTH: usize = TEST_GRAIN_SIZE_IN_SAMPLES as usize + 2;

// const TEST_GRAIN: Grain = Grain {
//     window: None,
//     window_parameter: None,
//     envelope_position: 0,
//     envelope_value: 0.0,

//     source: None,
//     source_length: None,
//     source_offset: None,
//     relative_position: 0,
//     source_position: 0,

//     grain_size: Some(TEST_GRAIN_SIZE_IN_SAMPLES),
//     finished: true,
//     id: 0,
// };

// #[test]
// fn check_bounds_envelope_sine() {
//     let mut g = TEST_GRAIN.clone();

//     // activate Sine envelope
//     g.window = Some(WindowFunction::Sine);
//     g.activate();

//     // create test buffer
//     let mut test_buffer: Vec<f32> = vec![];

//     // first sample is by definition 0
//     check!(
//         g.envelope_value == 0.0,
//         "First sample was not 0.0, but {}!",
//         g.envelope_value
//     );

//     // run through the grain until finished and store its values for further investigation
//     for _ in 0..TEST_BUFFER_LENGTH {
//         g.update_envelope();
//         test_buffer.push(g.envelope_value);
//     }

//     for sample in &test_buffer {
//         // check upper bound
//         check!(*sample <= 1.0, "Envelope grew bigger than 1!");

//         // check lower bound
//         check!(*sample >= 0.0, "Envelope got negative!");
//     }

//     // last sample in buffer has to be 0 if envelope is finished
//     check!(
//         test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
//         "Last sample is not 0!",
//     );
//     check!(g.is_finished(), "Envelope function was not finished yet!",);
// }

// #[test]
// fn check_bounds_envelope_hamming() {
//     let mut g = TEST_GRAIN.clone();

//     // activate Hamming envelope
//     g.window = Some(WindowFunction::Hamming);
//     g.activate();

//     // create test buffer
//     let mut test_buffer: Vec<f32> = vec![];

//     // first sample is by definition 0
//     check!(
//         g.envelope_value == 0.0,
//         "First sample was not 0.0, but {}!",
//         g.envelope_value
//     );

//     // run through the grain until finished and store its values for further investigation
//     for _ in 0..TEST_BUFFER_LENGTH {
//         g.update_envelope();
//         test_buffer.push(g.envelope_value);
//     }

//     for sample in &test_buffer {
//         // check upper bound
//         check!(*sample <= 1.0, "Envelope grew bigger than 1!");

//         // check lower bound
//         check!(*sample >= 0.0, "Envelope got negative!");
//     }

//     // last sample in buffer has to be 0 if envelope is finished
//     check!(
//         test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
//         "Last sample is not 0!",
//     );
//     check!(g.is_finished(), "Envelope function was not finished yet!",);
// }

// #[test]
// fn check_bounds_envelope_hann() {
//     let mut g = TEST_GRAIN.clone();

//     // activate Hann envelope
//     g.window = Some(WindowFunction::Hann);
//     g.activate();

//     // create test buffer
//     let mut test_buffer: Vec<f32> = vec![];

//     // first sample is by definition 0
//     check!(
//         g.envelope_value == 0.0,
//         "First sample was not 0.0, but {}!",
//         g.envelope_value
//     );

//     // run through the grain until finished and store its values for further investigation
//     for _ in 0..TEST_BUFFER_LENGTH {
//         g.update_envelope();
//         test_buffer.push(g.envelope_value);
//     }

//     for sample in &test_buffer {
//         // check upper bound
//         check!(*sample <= 1.0, "Envelope grew bigger than 1!");

//         // check lower bound
//         check!(*sample >= 0.0, "Envelope got negative!");
//     }

//     // last sample in buffer has to be 0 if envelope is finished
//     check!(
//         test_buffer[TEST_BUFFER_LENGTH - 1] == 0.0,
//         "Last sample is not 0!",
//     );
//     check!(g.is_finished(), "Envelope function was not finished yet!",);
// }
