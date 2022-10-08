#![allow(dead_code)]
#[derive(Debug)]
pub enum Source {
    AudioFile,
    DelayLine,
    Synthetic,
}

impl Source {
    pub fn get_source_sample_f32(&self, source_stream: *const [f32], position: f32) -> f32 {
        match self {
            Source::AudioFile => self.get_file_sample_f32_interpolated(source_stream, position),
            _ => 0.0,
        }
    }

    fn get_file_sample_f32_interpolated(&self, source_stream: *const [f32], position: f32) -> f32 {
        let trunc_position = position as usize;
        let first = unsafe { (*source_stream)[trunc_position] };
        let next = unsafe { (*source_stream)[trunc_position + 1] };

        (first + next) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    #[test]
    fn check_interpolation() {
        // create file stream
        let mut mock_file_stream: Vec<f32, 50> = Vec::new();

        // fill file stream with arbitrary numbers
        for i in 0..50 {
            mock_file_stream.push(i as f32 * 0.123456).unwrap();
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

        assert!(success);
    }
}
