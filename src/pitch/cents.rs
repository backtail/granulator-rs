use super::hz::{AudioFrequencies, Hz};

// CENT INTERVAL CONTS
pub const CENT_OCTAVE: u32 = 1200;
const TWELVE_TONE: ET = ET(12);
const TWELVE_TONE_STEP: f32 = CENT_OCTAVE as f32 / TWELVE_TONE.0 as f32;
const EMPTY_12TET_INTERVAL: CentsInterval = CentsInterval {
    cents: 0.0,
    tet: TWELVE_TONE,
    semitone_step: TWELVE_TONE_STEP,
    octave: 0,
    semitone: 0,
    rest: 0,
};

#[derive(Debug, Clone, Copy)]
pub struct ET(u32);

pub trait EqualTemperment {
    fn tet(&self) -> ET;
}

impl EqualTemperment for u32 {
    fn tet(&self) -> ET {
        if self < &2 {
            ET(2)
        } else {
            ET(self.clone())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CentsInterval {
    pub cents: f32,
    tet: ET,
    semitone_step: f32,
    pub octave: i32,
    pub semitone: i32,
    pub rest: i32,
}

impl CentsInterval {
    // Constrcutors
    pub fn zero() -> CentsInterval {
        EMPTY_12TET_INTERVAL
    }
    pub fn new(cents: f32) -> CentsInterval {
        CentsInterval {
            cents,
            tet: TWELVE_TONE,
            semitone_step: TWELVE_TONE_STEP,
            octave: (cents / CENT_OCTAVE as f32) as i32,
            semitone: (cents / TWELVE_TONE_STEP) as i32 % TWELVE_TONE.0 as i32,
            rest: (cents - TWELVE_TONE_STEP * (cents / TWELVE_TONE_STEP).trunc()) as i32,
        }
    }

    pub fn new_microtonal(cents: f32, equal_temperment: ET) -> CentsInterval {
        let tet = equal_temperment;
        let semitone_step = CENT_OCTAVE as f32 / tet.0 as f32;
        CentsInterval {
            tet,
            cents,
            semitone_step,
            octave: (cents / CENT_OCTAVE as f32) as i32,
            semitone: (cents / semitone_step) as i32 % tet.0 as i32,
            rest: (cents - semitone_step * (cents / semitone_step).trunc()) as i32,
        }
    }

    pub fn octave_only(&mut self) -> CentsInterval {
        self.semitone = 0;
        self.rest = 0;
        *self
    }

    pub fn semitone_only(&mut self) -> CentsInterval {
        self.octave = 0;
        self.rest = 0;
        *self
    }

    pub fn rest_only(&mut self) -> CentsInterval {
        self.octave = 0;
        self.semitone = 0;
        *self
    }

    pub fn to_hz(&self, from: Hz) -> Hz {
        ((self.f32() / CENT_OCTAVE as f32).exp2() * from.0).hz()
    }

    fn f32(&self) -> f32 {
        (self.octave as i32 * CENT_OCTAVE as i32
            + self.semitone as i32 * (CENT_OCTAVE as f32 / self.tet.0 as f32) as i32
            + self.rest as i32) as f32
    }

    pub fn add_octave(&mut self, n: i32) -> CentsInterval {
        self.octave += n;
        self.calc_cents();
        *self
    }

    pub fn add_semitone(&mut self, n: i32) -> CentsInterval {
        let max_semitones = self.tet.0 as i32;
        if n.abs() >= max_semitones {
            self.octave += n / max_semitones;
            self.semitone += n % max_semitones;
        } else {
            self.semitone += n;
        }
        self.calc_cents();

        *self
    }

    pub fn add_rest(&mut self, n: i32) -> CentsInterval {
        let max_rest = self.semitone_step as i32;
        if n.abs() >= max_rest {
            self.semitone += n / max_rest;
            self.rest += n % max_rest;
        } else {
            self.rest += n;
        }
        self.calc_cents();

        *self
    }

    fn calc_cents(&mut self) {
        self.cents = (self.octave * CENT_OCTAVE as i32) as f32
            + self.semitone as f32 * self.semitone_step
            + self.rest as f32;
    }

    pub fn ratio(&self) -> f32 {
        (self.cents / CENT_OCTAVE as f32).exp2()
    }
}
