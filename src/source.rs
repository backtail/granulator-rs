#[derive(Clone, Copy, Debug)]
pub enum Source {
    AudioFile,
    DelayLine,
    Synthetic,
}

impl Source {
    pub fn get_source_sample_f32(&self, source_stream: &[f32], position: f32) -> f32 {
        match self {
            Source::AudioFile => self.get_file_sample_f32_interpolated(source_stream, position),
            _ => 0.0,
        }
    }

    fn get_file_sample_f32_interpolated(&self, source_stream: &[f32], position: f32) -> f32 {
        let trunc_position = position as usize;
        let first = source_stream[trunc_position];
        let next = source_stream[trunc_position + 1];

        (first + next) * 0.5
    }
}
