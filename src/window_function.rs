use super::audio_tools::{PI, TWO_PI};

#[allow(unused_imports)]
use micromath::F32Ext;

/// Currently only `Sine`, `Hann` and `Hamming` have implementations.
///
/// Other window functions will always output `0.0` as envelope amplitude!
#[derive(Debug, Clone, Copy)]
pub enum WindowFunction {
    Trapezodial,
    Gaussian,
    Sine,
    Hann,
    Hamming,
    Tukey,
}

impl WindowFunction {
    fn sine_function(&self, position: f32, size: f32) -> f32 {
        ((PI * position) / size).sin()
    }

    fn hann_function(&self, position: f32, size: f32) -> f32 {
        0.5 * (1.0 - (TWO_PI * position / size).cos())
    }

    fn hamming_function(&self, position: f32, size: f32) -> f32 {
        0.54 * (0.46 - (TWO_PI * position / size).cos())
    }

    fn gaussian_function(&self, position: f32, size: f32, parameter: f32) -> f32 {
        let sigma = 0.5 * (parameter + 0.01);

        (((position - size / 2.0) / (sigma * size / 2.0)).powf(2.0) * -0.5).exp()
    }

    fn tukey_function(&self, position: f32, size: f32, parameter: f32) -> f32 {
        let truncation = 2.5 * (parameter + 0.01);

        let value = 1.0 / (2.0 * truncation) * (1.0 - (TWO_PI * position / size).cos());
        value.clamp(0.0, 1.0)
    }

    pub fn get_envelope_value(&self, position: f32, size: f32, parameter: f32) -> f32 {
        match self {
            WindowFunction::Sine => self.sine_function(position, size),
            WindowFunction::Hann => self.hann_function(position, size),
            WindowFunction::Hamming => self.hamming_function(position, size),
            WindowFunction::Gaussian => self.gaussian_function(position, size, parameter),
            WindowFunction::Tukey => self.tukey_function(position, size, parameter),
            _ => 0.0,
        }
    }
}
