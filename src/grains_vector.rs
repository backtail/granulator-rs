#![allow(dead_code)]

use super::grain::Grain;
use super::grain::WindowFunction;
use super::manager::MAX_GRAINS;
use super::pointer_wrapper::BufferSlice;

use heapless::Vec;
use num_traits::AsPrimitive;

#[derive(Debug)]
pub struct GrainsVector<T: AsPrimitive<f32>> {
    grains: Vec<Grain<T>, MAX_GRAINS>,
}

impl<T: AsPrimitive<f32>> GrainsVector<T> {
    pub fn new() -> Self {
        GrainsVector { grains: Vec::new() }
    }

    pub fn push_grain(
        &mut self,
        id: usize,
        sub_slice: BufferSlice<T>,
        window: WindowFunction,
        window_param: f32,
        pitch: f32,
        velocity: f32,
    ) -> Result<(), usize> {
        if self
            .grains
            .push(Grain::new(
                id,
                sub_slice,
                window,
                window_param,
                pitch,
                velocity,
            ))
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
        let mut sample = 0.0;
        for grain in &mut self.grains {
            sample += grain.get_next_sample();
        }
        sample
    }

    pub fn get_grains(&self) -> &Vec<Grain<T>, MAX_GRAINS> {
        &self.grains
    }

    pub fn get_mut_grains(&mut self) -> &mut Vec<Grain<T>, MAX_GRAINS> {
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
                1.0,
                1.0,
            )
            .unwrap();
        }

        g.flush();

        assert!(g.grains.len() == 0);
    }
}
