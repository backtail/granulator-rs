use micromath::F32Ext;

pub const PI: f32 = 3.141592653589793;
pub const TWO_PI: f32 = 6.2831853071795864;

pub fn soft_clip(sample: f32) -> f32 {
    sample.atan_norm()
}
