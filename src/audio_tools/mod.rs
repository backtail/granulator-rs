use micromath::F32Ext;

pub const PI: f32 = 3.141592653589793;
pub const TWO_PI: f32 = 6.2831853071795864;

pub fn soft_clip(sample: f32) -> f32 {
    sample.atan_norm()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;
    use heapless::Vec;

    #[test]
    fn check_bounds() {
        let mut sample_buffer: Vec<f32, 10> = Vec::new();
        for i in 0..10 {
            // range from -2 to 2
            let value = ((i - 5) as f32) / 5.0;
            sample_buffer.push(value).unwrap();
        }

        for sample in sample_buffer {
            let processed = soft_clip(sample);
            check!(processed <= (1.0 + f32::EPSILON));
            check!(processed >= (-1.0 - f32::EPSILON));
        }
    }
}
