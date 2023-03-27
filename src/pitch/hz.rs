use super::cents::{CentsInterval, CENT_OCTAVE, ET};

use core::f32::consts::LN_2;

#[allow(unused)]
use micromath::F32Ext;

// FREQUENCY CONSTS
const MIN_HZ: f32 = 16.35; // C0
const MAX_HZ: f32 = 7902.13; // B8

#[derive(Debug, Clone, Copy)]
pub struct Hz(pub f32);

impl Hz {
    #[allow(unused)]
    pub fn interval(&self, to: Hz) -> CentsInterval {
        CentsInterval::new(CENT_OCTAVE as f32 * ((to.0 / self.0).ln()) / LN_2)
    }

    #[allow(unused)]
    pub fn microtonal_interval(&self, to: Hz, tet: ET) -> CentsInterval {
        CentsInterval::new_microtonal(CENT_OCTAVE as f32 * ((to.0 / self.0).ln()) / LN_2, tet)
    }
}
pub trait AudioFrequencies {
    fn hz(&self) -> Hz;
}

impl AudioFrequencies for f32 {
    fn hz(&self) -> Hz {
        Hz(self.clamp(MIN_HZ, MAX_HZ))
    }
}

// #[derive(Clone, Copy)]
// pub enum Letter {
//     A,
//     B,
//     C,
//     D,
//     E,
//     F,
//     G,
// }

// #[derive(Clone, Copy)]
// pub enum Accidental {
//     DoubleFlat = -2,
//     Flat = -1,
//     Natural = 0,
//     Sharp = 1,
//     DoubleSharp = 2,
// }

// #[derive(Clone, Copy)]
// pub enum Degree {
//     Tonic = 1,
//     Supertonic = 2,
//     Mediant = 3,
//     Subdominant = 4,
//     Dominant = 5,
//     Submediant = 6,
//     Subtonic = 7,
// }

// #[derive(Clone, Copy)]
// pub struct ScientificNote {
//     pub letter: Letter,
//     pub accidental: Accidental,
//     pub octave: i32,
// }

// impl ScientificNote {
//     pub fn new(letter: Letter, accidental: Accidental, octave: i32) -> ScientificNote {
//         ScientificNote {
//             octave,
//             letter,
//             accidental,
//         }
//     }
// }
