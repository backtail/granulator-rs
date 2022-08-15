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
}
