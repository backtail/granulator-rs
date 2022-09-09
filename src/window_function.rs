use micromath::F32;

use super::audio_tools::{PI, TWO_PI};

#[derive(Clone, Copy, Debug)]
pub enum WindowFunction {
    Trapezodial { slope: f32 },
    Gaussian { sigma: f32 },
    Sine,
    Hann,
    Hamming,
    Tukey { truncation_height: f32 },
}

impl WindowFunction {
    fn sine_function(&self, position: f32, size: f32) -> f32 {
        F32::sin(F32::from((PI * position) / size)).0
    }

    fn hann_function(&self, position: f32, size: f32) -> f32 {
        0.5 * (1.0 - F32::cos(F32::from((TWO_PI * position) / size)).0)
    }

    fn hamming_function(&self, position: f32, size: f32) -> f32 {
        0.54 - 0.46 * F32::cos(F32::from((TWO_PI * position) / size)).0
    }

    pub fn get_envelope_value(&self, position: f32, size: f32, _parameter: Option<f32>) -> f32 {
        match self {
            WindowFunction::Sine => self.sine_function(position, size),
            WindowFunction::Hann => self.hann_function(position, size),
            WindowFunction::Hamming => self.hamming_function(position, size),
            _ => 0.0,
        }
    }
}
