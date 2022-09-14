use crate::grain::Grain;
use crate::manager::MAX_GRAINS;
use heapless::Vec;

pub struct GrainsVector {
    grains: Vec<Grain, MAX_GRAINS>,
}

impl GrainsVector {
    pub fn new() -> Self {
        GrainsVector { grains: Vec::new() }
    }

    pub fn push_grain(&mut self, id: usize, sub_slice: *const [f32]) -> Result<(), usize> {
        if self.grains.push(Grain::new(id, sub_slice)).is_err() {
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

    pub fn is_finished(&self, id: usize) -> Result<bool, usize> {
        for grain in &self.grains {
            if grain.id == id {
                return Ok(grain.finished);
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
    use assert2::*;

    #[test]
    fn test_new_grains() {
        let p = [0_f32; 10].as_ptr();
        let slice_p = core::ptr::slice_from_raw_parts(p, 10);
        let mut g = GrainsVector::new();

        g.push_grain(0, slice_p).unwrap();

        check!(g.grains.len() == 1);

        g.remove_grain(0).unwrap();

        check!(g.grains.len() == 0);
    }
}
