#![allow(dead_code)]

use crate::source::Source;
use crate::window_function::WindowFunction;

use super::grain::Grain;
use super::manager::MAX_GRAINS;
use super::pointer_wrapper::BufferSlice;
use heapless::Vec;

#[derive(Debug)]
pub struct GrainsVector {
    grains: Vec<Grain, MAX_GRAINS>,
}

impl GrainsVector {
    pub fn new() -> Self {
        GrainsVector { grains: Vec::new() }
    }

    pub fn push_grain(
        &mut self,
        id: usize,
        sub_slice: BufferSlice,
        window: WindowFunction,
        source: Source,
        pitch: f32,
        velocity: f32,
    ) -> Result<(), usize> {
        if self
            .grains
            .push(Grain::new(id, sub_slice, window, source, pitch, velocity))
            .is_err()
        {
            Err(id)
        } else {
            Ok(())
        }
    }

    pub fn remove_grain(&mut self, id: usize) -> Result<(), usize> {
        for (vector_id, grain) in self.grains.iter_mut().enumerate() {
            if grain.id == id {
                self.grains.remove(vector_id);
                return Ok(());
            }
        }

        Err(id)
    }

    pub fn flush(&mut self) {
        self.grains.clear();
    }

    // will always be processed in highest priority, therefore no Result
    pub fn get_next_sample(&mut self) -> f32 {
        let mut sample = 0_f32;
        for grain in &mut self.grains {
            sample += grain.get_next_sample();
        }
        sample
    }

    pub fn get_grains(&self) -> &Vec<Grain, MAX_GRAINS> {
        &self.grains
    }

    pub fn get_mut_grains(&mut self) -> &mut Vec<Grain, MAX_GRAINS> {
        &mut self.grains
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLICE: [f32; 10] = [0_f32; 10];

    #[test]
    fn push_and_remove_a_grain() {
        let mut g = GrainsVector::new();

        g.push_grain(
            0,
            BufferSlice::from_slice(&SLICE),
            WindowFunction::Sine,
            Source::AudioFile,
            1.0,
            1.0,
        )
        .unwrap();

        assert!(g.grains.len() == 1);

        g.remove_grain(0).unwrap();

        assert!(g.grains.len() == 0);
    }

    #[test]
    fn get_a_grain() {
        let mut g = GrainsVector::new();

        g.push_grain(
            0,
            BufferSlice::from_slice(&SLICE),
            WindowFunction::Sine,
            Source::AudioFile,
            1.0,
            1.0,
        )
        .unwrap();

        assert!(g.get_grains().len() == 1);
        assert!(g.get_mut_grains().len() == 1);
    }

    #[test]
    fn flush_vector() {
        let mut g = GrainsVector::new();

        for i in 0..MAX_GRAINS {
            g.push_grain(
                i,
                BufferSlice::from_slice(&SLICE),
                WindowFunction::Sine,
                Source::AudioFile,
                1.0,
                1.0,
            )
            .unwrap();
        }

        g.flush();

        assert!(g.grains.len() == 0);
    }
}
