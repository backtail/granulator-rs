use super::cents::CentsInterval;

use ClassicalIntervals::*;

/// Statically allocated scales
static SCALE_INTERVALS: [HeptatonicSequence; 4] = [
    [Whole, Whole, Half, Whole, Whole, Whole, Half],
    [Whole, Half, Whole, Whole, Whole, Whole, Half],
    [Whole, Whole, Half, Whole, Half, MinorThird, Half],
    [Whole, Half, Whole, Whole, Half, MinorThird, Half],
];

pub fn get_ratios_for(scale: ScaleType, mode: ModeType) -> HeptatonicRatios {
    let mut ratios = [0_f32; HEPTATONIC];
    for (i, semitone) in get_semitones_for(scale, mode).iter().enumerate() {
        ratios[i] = CentsInterval::zero().add_semitone(*semitone as i32).ratio();
    }

    ratios
}

pub fn get_semitones_for(scale: ScaleType, mode: ModeType) -> HeptatonicSemitones {
    let mut semitones = [0_u32; HEPTATONIC];
    let mut iter = SCALE_INTERVALS[scale as usize]
        .iter()
        .cycle()
        .skip(mode as usize);
    let mut semitone = 0;

    for step in semitones.iter_mut().skip(1) {
        semitone += *iter.next().unwrap() as u32;
        *step = semitone;
    }

    semitones
}

/// Heptatonic scales always have 7 notes
pub const HEPTATONIC: usize = 7;
pub type HeptatonicSequence = [ClassicalIntervals; HEPTATONIC];
pub type HeptatonicRatios = [f32; HEPTATONIC];
pub type HeptatonicSemitones = [u32; HEPTATONIC];

/// Types of classical tone intervals
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum ClassicalIntervals {
    Unison = 0,
    Half = 1,
    Whole = 2,
    MinorThird = 3,
    MajorThird = 4,
    PerfectFourth = 5,
    Tritone = 6,
    PerfectFifth = 7,
    MinorSixth = 8,
    MajorSixth = 9,
    MinorSeventh = 10,
    MajorSeventh = 11,
    Octave = 12,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum ModeType {
    /// Major
    Ionian = 0,
    Dorian = 1,
    Phrygian = 2,
    Lydian = 3,
    Mixolydian = 4,
    /// Natural Minor
    Aeolian = 5,
    Locrian = 6,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum ScaleType {
    Diatonic = 0,
    Melodic = 1,
    HarmonicMinor = 2,
    MarmonicMajor = 3,
}
