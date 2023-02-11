use micromath::F32Ext;

pub fn soft_clip(sample: f32) -> f32 {
    sample.atan_norm()
}

#[cfg(test)]
mod tests {
    use super::*;
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
            assert!(processed <= (1.0 + f32::EPSILON));
            assert!(processed >= (-1.0 - f32::EPSILON));
        }
    }
}
