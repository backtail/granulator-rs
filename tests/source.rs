use assert2::*;
use granulator::source::Source;

#[test]
fn check_interpolation() {
    // create file stream
    let mut mock_file_stream: Vec<f32> = vec![];

    // fill file stream with arbitrary numbers
    for i in 0..50 {
        mock_file_stream.push(i as f32 * 0.123456);
    }

    // convert it to slice
    let mock_file_stream = mock_file_stream.as_slice();

    // setup audio file mockup
    let source_file = Source::AudioFile;

    // create f32 offset value
    let offset = 42.314;

    // calculate interolated value
    let output = source_file.get_source_sample_f32(mock_file_stream, offset);

    // convert offset to integer
    let offset = offset as usize;

    // sign of gradient dictates compare logic
    let gradient = mock_file_stream[offset] - mock_file_stream[offset + 1];

    let mut success = false;

    if gradient.is_sign_negative() {
        if output > mock_file_stream[offset] && output < mock_file_stream[offset + 1] {
            success = true;
        }
    } else {
        if output < mock_file_stream[offset] && output > mock_file_stream[offset + 1] {
            success = true;
        }
    }

    check!(
        success,
        "{} should be between {} and {}",
        output,
        mock_file_stream[offset],
        mock_file_stream[offset + 1]
    );
}
