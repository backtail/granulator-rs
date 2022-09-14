use super::manager::FS;
use super::source::Source;
use super::window_function::WindowFunction;

pub struct Grain {
    // envelope variables
    pub window: WindowFunction,
    pub window_parameter: Option<f32>,
    pub envelope_position: f32, // between 0..grain_length (in samples)
    pub envelope_value: f32,    // between 0..1

    // source variables
    pub source: Source,
    pub source_material: *const [f32], // slice as pointer
    pub source_position: f32,          // between 0..grain_length (in samples)
    pub source_value: f32,             // between 0..1
    pub pitch: f32,

    // grain variables
    pub grain_length: f32, // in samples
    pub finished: bool,

    // misc
    pub id: usize,
}

impl Grain {
    pub fn new(id: usize, source_material: *const [f32]) -> Self {
        Grain {
            window: WindowFunction::Sine,
            window_parameter: None,
            envelope_position: 0.0,
            envelope_value: 0.0,

            source: Source::AudioFile,
            source_material,
            source_position: 0.0,
            source_value: 0.0,
            pitch: 1.0,

            grain_length: unsafe { (*source_material).len() } as f32,
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

    pub fn update_envelope(&mut self) -> f32 {
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

        self.envelope_value
    }

    pub fn update_source_sample(&mut self) -> f32 {
        if !self.finished {
            // move playhead
            self.source_position += self.pitch;

            // wrap around
            if self.source_position >= self.grain_length - 1.0 {
                self.source_position -= self.grain_length;
            }

            // interpolate source value
            self.source_value = self
                .source
                .get_source_sample_f32(self.source_material, self.source_position);
        }

        self.source_value
    }

    pub fn get_next_sample(&mut self) -> f32 {
        self.update_envelope() * self.update_source_sample()
    }
}

/// dead code
///
/// helps with pointer concept
fn _test() {
    let mut data = [0_f32; 1000];
    for i in 0..1000 {
        data[i] = i as f32;
    }

    let slice = data.as_slice();
    let buffer_pointer = &slice[0] as *const f32;

    // create pointer with offset
    let pointer_offset = unsafe { buffer_pointer.offset(9) };

    // create slice from pointer position
    let slice_pointer = core::ptr::slice_from_raw_parts(pointer_offset, 10);

    // return value from sllice
    let _value = unsafe { (*slice_pointer)[3] };
}
