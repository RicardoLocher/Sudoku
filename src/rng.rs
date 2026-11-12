pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            0
        } else {
            self.next_u64() % bound
        }
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.below((i + 1) as u64) as usize;
            slice.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn below_is_in_range() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..1000 {
            assert!(rng.below(9) < 9);
        }
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut rng = SplitMix64::new(1);
        let mut values: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        rng.shuffle(&mut values);
        values.sort_unstable();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
