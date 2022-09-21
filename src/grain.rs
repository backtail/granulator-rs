use super::pointer_wrapper::BufferSlice;
use super::source::Source;
use super::window_function::WindowFunction;

#[derive(Debug)]
pub struct Grain {
    // envelope variables
    pub window: WindowFunction,
    pub window_parameter: Option<f32>,
    pub envelope_position: f32, // between 0..grain_length (in samples)
    pub envelope_value: f32,    // between 0..1

    // source variables
    pub source: Source,
    pub source_sub_slice: BufferSlice, // slice as pointer
    pub source_position: f32,          // between 0..grain_length (in samples)
    pub source_value: f32,             // between 0..1
    pub pitch: f32,

    // grain variables
    pub finished: bool,

    // misc
    pub id: usize,
}

impl Grain {
    pub fn new(
        id: usize,
        source_sub_slice: BufferSlice,
        window: WindowFunction,
        source: Source,
        pitch: f32,
    ) -> Self {
        Grain {
            window,
            window_parameter: None,
            envelope_position: 0.0,
            envelope_value: 0.0,

            source,
            source_sub_slice,
            source_position: 0.0,
            source_value: 0.0,
            pitch,

            finished: false,

            id,
        }
    }

    pub fn update_envelope(&mut self) -> f32 {
        if !self.finished {
            // calcualte new value
            self.envelope_value = self.window.get_envelope_value(
                self.envelope_position,
                self.source_sub_slice.length,
                self.window_parameter,
            );

            // finish grain if it reaches end
            if self.envelope_position < self.source_sub_slice.length {
                self.envelope_position += 1.0;
            } else {
                self.finished = true;
                self.envelope_value = 0.0;
            }
        }

        self.envelope_value
    }

    pub fn update_source_sample(&mut self) -> f32 {
        if !self.finished {
            // move playhead
            self.source_position += self.pitch;

            // wrap around
            if self.source_position >= self.source_sub_slice.length - 1.0 {
                self.source_position -= self.source_sub_slice.length;
            }

            // interpolate source value
            self.source_value = self
                .source
                .get_source_sample_f32(self.source_sub_slice.as_slice(), self.source_position);
        }

        self.source_value
    }

    pub fn get_next_sample(&mut self) -> f32 {
        self.update_envelope() * self.update_source_sample()
    }
}
