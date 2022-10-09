use oorandom::Rand32;

/// Returns a random number between [-1.0, 1.0).
pub fn get_random_bipolar_float(rng: &mut Rand32) -> f32 {
    rng.rand_float() * 2.0 - 1.0
}

/// Returns a random number between [0.0, 1.0).
pub fn get_random_unipolar_float(rng: &mut Rand32) -> f32 {
    rng.rand_float()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_bounds() {
        let seed = 0;
        let address = core::ptr::addr_of!(seed);
        let mut rng = Rand32::new(address as u64);
        let random = get_random_float(&mut rng);
        for _ in 0..1000 {
            assert!(random <= 1.0);
            assert!(random >= -1.0);
        }
    }
}
