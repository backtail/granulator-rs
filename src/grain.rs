use super::manager::FS;
use super::source::Source;
use super::window_function::WindowFunction;

#[derive(Clone, Copy, Debug)]
pub struct Grain<'a> {
    // envelope variables
    pub window: WindowFunction,
    pub window_parameter: Option<f32>,
    pub envelope_position: f32, // between 0..grain_length (in samples)
    pub envelope_value: f32,    // between 0..1

    // source variables
    pub source: Source,
    pub source_material: &'a [f32],
    pub source_position: f32, // between 0..grain_length (in samples)
    pub source_value: f32,    // between 0..1

    // grain variables
    pub grain_length: f32, // in samples
    pub finished: bool,

    // misc
    pub id: usize,
}

impl<'a> Grain<'a> {
    pub fn new(id: usize, source_material: &'a [f32]) -> Self {
        Grain {
            window: WindowFunction::Sine,
            window_parameter: None,
            envelope_position: 0.0,
            envelope_value: 0.0,

            source: Source::AudioFile,
            source_material,
            source_position: 0.0,
            source_value: 0.0,

            grain_length: source_material.len() as f32,
            finished: false,

            id,
        }
    }

    pub fn set_grain_size(&mut self, size_in_ms: f32) {
        self.grain_length = (FS as f32 * size_in_ms) / 1000.0;
    }

    pub fn get_grain_size_in_samples(&self) -> usize {
        self.grain_length as usize
    }

    pub fn update_envelope(&mut self) {
        if !self.finished {
            // calcualte new value
            self.envelope_value = self.window.get_envelope_value(
                self.envelope_position,
                self.grain_length,
                self.window_parameter,
            );

            // finish grain if it reaches end
            if self.envelope_position < self.grain_length {
                self.envelope_position += 1.0;
            } else {
                self.finished = true;
                self.envelope_value = 0.0;
            }
        }
    }

    pub fn update_source_sample(&mut self) {
        if !self.finished {
            // move playhead
            self.source_position += 1.0;

            // wrap around
            if self.source_position >= self.grain_length - 1.0 {
                self.source_position -= self.grain_length;
            }

            // interpolate source value
            self.source_value = self
                .source
                .get_source_sample_f32(self.source_material, self.source_position);
        }
    }
}
