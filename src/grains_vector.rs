use crate::source::Source;
use crate::window_function::WindowFunction;

use super::grain::Grain;
use super::manager::MAX_GRAINS;
use super::pointer_wrapper::BufferSlice;
use heapless::Vec;
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
    ) -> Result<(), usize> {
        if self
            .grains
            .push(Grain::new(id, sub_slice, window, source))
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
    use crate::pointer_wrapper::BufferPointer;

    use super::*;
    use assert2::*;

    #[test]
    fn test_new_grains() {
        let mut g = GrainsVector::new();

        let slice_p = BufferSlice {
            ptr: BufferPointer(core::ptr::null()),
            length: 1.0,
        };

        g.push_grain(0, slice_p, WindowFunction::Sine, Source::AudioFile)
            .unwrap();

        check!(g.grains.len() == 1);

        g.remove_grain(0).unwrap();

        check!(g.grains.len() == 0);
    }
}
