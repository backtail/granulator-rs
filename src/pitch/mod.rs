pub(crate) mod cents;
pub(crate) mod hz;
pub(crate) mod scales;

pub use scales::HeptatonicRatios;
pub use scales::{get_ratios_for, get_semitones_for};
pub use scales::{ModeType, ScaleType};

use core::ops::Neg;
#[allow(unused)]
use micromath::F32Ext;

pub trait Ratio {
    fn autotune_to(&self, sequence: Option<HeptatonicRatios>) -> f32;
}

impl Ratio for f32 {
    #[inline(always)]
    fn autotune_to(&self, sequence: Option<HeptatonicRatios>) -> f32 {
        if let Some(s) = sequence {
            if *self > 1.0 {
                let octave = self.log2().trunc() + 1.0;
                let scaled_sequence = s.map(|f| f * octave);
                find_nearest_value(*self, scaled_sequence)
            } else if *self < 1.0 {
                let octave = self.log2().neg().trunc() + 1.0;
                let scaled_sequence = s.map(|f| f / (octave + 1.0));
                find_nearest_value(*self, scaled_sequence)
            } else {
                1.0
            }
        } else {
            *self
        }
    }
}

fn find_nearest_value(value: f32, sequence: HeptatonicRatios) -> f32 {
    let mut output = 0.0;
    for (i, ratio) in sequence.iter().enumerate() {
        if value <= *ratio {
            output = sequence[i];
            break;
        } else {
            output = sequence[0] * 2.0;
        }
    }

    output
}
