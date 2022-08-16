#[derive(Clone, Copy)]
pub enum Source {
    AudioFile,
    DelayLine,
    Synthetic,
}

impl Source {
    fn get_file_sample(&self, position: usize, offset: usize) -> usize {
        offset + position
    }

    pub fn get_source_sample(&self, position: usize, offset: usize) -> usize {
        match self {
            Source::AudioFile => self.get_file_sample(position, offset),
            _ => offset,
        }
    }

    pub fn get_source_sample_f32(&self, source_stream: &[f32], offset: f32) -> f32 {
        match self {
            Source::AudioFile => self.get_file_sample_f32_interpolated(source_stream, offset),
            _ => 0.0,
        }
    }

    fn get_file_sample_f32_interpolated(&self, source_stream: &[f32], offset: f32) -> f32 {
        let trunc_offset = offset as usize;
        let first = source_stream[trunc_offset];
        let next = source_stream[trunc_offset + 1];

        (first + next) * 0.5
    }
}
