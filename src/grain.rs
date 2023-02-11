use super::pointer_wrapper::BufferSlice;

use core::f32::consts::{PI, TAU};
use num_traits::AsPrimitive;

#[allow(unused_imports)]
use micromath::F32Ext;

/// All possible window functions that can be applied to a given audio source
#[derive(Debug, Clone, Copy)]
pub enum WindowFunction {
    Trapezodial,
    Gaussian,
    Sine,
    Hann,
    Hamming,
    Tukey,
}

#[derive(Debug)]
pub struct Grain<T: AsPrimitive<f32>> {
    // envelope variables
    pub window: WindowFunction,
    pub window_param: f32,
    pub envelope_position: f32, // between 0..grain_length (in samples)
    pub envelope_value: f32,    // between 0..1

    // source variables
    pub source_sub_slice: BufferSlice<T>, // slice as pointer of any numeric type
    pub source_position: f32,             // between 0..grain_length (in samples)
    pub source_value: f32,                // between 0..1

    pub pitch: f32,
    pub velocity: f32,

    // grain variables
    pub finished: bool,

    // misc
    pub id: usize,
}

impl<T: AsPrimitive<f32>> Grain<T> {
    pub fn new(
        id: usize,
        source_sub_slice: BufferSlice<T>,
        window: WindowFunction,
        window_param: f32,
        pitch: f32,
        velocity: f32,
    ) -> Self {
        Grain {
            window,
            window_param,
            envelope_position: 0.0,
            envelope_value: 0.0,

            source_sub_slice,
            source_position: 0.0,
            source_value: 0.0,

            pitch,
            velocity,

            finished: false,

            id,
        }
    }

    fn get_envelope_value(&self) -> f32 {
        let position = self.source_sub_slice.length as f32;
        match self.window {
            WindowFunction::Sine => ((PI * self.envelope_position) / position).sin(),
            WindowFunction::Hann => 0.5 * (1.0 - (TAU * self.envelope_position / position).cos()),
            WindowFunction::Hamming => {
                0.54 * (0.46 - (TAU * self.envelope_position / position).cos())
            }
            WindowFunction::Gaussian => {
                // window parameter
                let sigma = 0.5 * (self.window_param + 0.01);

                (((self.envelope_position - position / 2.0) / (sigma * position / 2.0)).powf(2.0)
                    * -0.5)
                    .exp()
            }

            WindowFunction::Tukey => {
                // window parameter
                let truncation = 2.5 * (self.window_param + 0.01);

                let value = 1.0 / (2.0 * truncation)
                    * (1.0 - (TAU * self.envelope_position / position).cos());
                value.clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }

    fn get_source_sample_interpolated(&self, source_stream: &BufferSlice<T>, position: f32) -> f32 {
        let first = source_stream.get_f32_value_at(position as usize);
        let next = source_stream.get_f32_value_at(position as usize + 1);
        (first + next) * 0.5
    }

    pub fn update_envelope(&mut self) -> f32 {
        if !self.finished {
            // calcualte new value
            self.envelope_value = self.get_envelope_value();

            // finish grain if it reaches end
            if self.envelope_position < self.source_sub_slice.length as f32 {
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
            if self.source_position >= self.source_sub_slice.length as f32 - 1.0 {
                self.source_position -= self.source_sub_slice.length as f32;
            }

            // interpolate source value
            self.source_value =
                self.get_source_sample_interpolated(&self.source_sub_slice, self.source_position);
        }

        self.source_value
    }

    pub fn get_next_sample(&mut self) -> f32 {
        self.update_envelope() * self.update_source_sample() * self.velocity
    }
}
