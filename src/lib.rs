// 8^20 + 3, an arbitrary number that provides an acceptable cycle length
const MULTIPLIER: u64 = 0x1000000000000003;
const BYTE_LEN: usize = 8;

#[derive(Default)]
pub struct PcgSeed(pub [u8; BYTE_LEN]);

use rand_core::*;
use std::num::Wrapping;

pub struct Pcg {
    state: u64,
}

impl Pcg {
    #[cfg(test)]
    pub fn get_state(&self) -> u64 {
        self.state
    }

    pub fn skip(&mut self, n: i32) {
        let mut x = Wrapping(self.state);
        for _ in 0..n {
            x *= Wrapping(MULTIPLIER);
        }
        self.state = x.0;
    }
}

impl RngCore for Pcg {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.state = (Wrapping(self.state) * Wrapping(MULTIPLIER)).0;
        (self.state ^ (self.state >> 22)) >> (22 + (self.state >> 61))
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl SeedableRng for Pcg {
    type Seed = PcgSeed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self::seed_from_u64(arr_to_u64(seed))
    }

    fn seed_from_u64(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl AsMut<[u8]> for PcgSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

fn arr_to_u64(mut arr: PcgSeed) -> u64 {
    let mut seed: u64 = 0;
    let mutarr = PcgSeed::as_mut(&mut arr);
    for i in 0..(BYTE_LEN) {
        seed ^= (mutarr[i] as u64) << 8 * i;
    }
    seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seed() {
        let arr = [0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01];
        let pcg = Pcg::from_seed(PcgSeed(arr));
        assert_eq!(pcg.get_state(), 0x0123456789abcdef);
    }

    #[test]
    fn test_seed_from_u64() {
        let seed = u64::MAX;
        let pcg = Pcg::seed_from_u64(seed);
        assert_eq!(pcg.get_state(), seed);
    }

    #[test]
    fn test_next_u64() {
        let seed = rand::random::<u64>();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER)).0;
        let next = (state ^ (state >> 22)) >> (22 + (state >> 61));

        let mut pcg = Pcg::seed_from_u64(seed);
        assert_eq!(pcg.next_u64(), next);
    }

    #[test]
    fn test_next_u32() {
        let seed = rand::random::<u64>();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER)).0;
        let next = ((state ^ (state >> 22)) >> (22 + (state >> 61))) as u32;

        let mut pcg = Pcg::seed_from_u64(seed);
        assert_eq!(pcg.next_u32(), next);
    }

    #[test]
    fn test_fill_bytes() {
        let seed = rand::random::<u64>();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER)).0;
        let next = (state ^ (state >> 22)) >> (22 + (state >> 61));
        let secondstate = (Wrapping(state) * Wrapping(MULTIPLIER)).0;
        let secondnext = (secondstate ^ (secondstate >> 22)) >> (22 + (secondstate >> 61));
        let mut next_sixteen_expected_bytes = [0; 16];
        for i in 0..8 {
            next_sixteen_expected_bytes[i] = ((next >> 8 * i) % 256) as u8;
        }
        for i in 0..8 {
            next_sixteen_expected_bytes[i + 8] = ((secondnext >> 8 * i) % 256) as u8;
        }

        let mut arr = [0; 16];
        let mut pcg = Pcg::seed_from_u64(seed);
        pcg.fill_bytes(&mut arr);
        assert_eq!(arr, next_sixteen_expected_bytes);

        pcg = Pcg::seed_from_u64(seed);
        assert!(pcg.try_fill_bytes(&mut arr).is_ok());
        assert_eq!(arr, next_sixteen_expected_bytes);
    }

    #[test]
    fn test_skip() {
        let seed = rand::random::<u64>();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER) * Wrapping(MULTIPLIER)).0;
        let next = (state ^ (state >> 22)) >> (22 + (state >> 61));

        let mut pcg = Pcg::seed_from_u64(seed);
        pcg.skip(1);
        assert_eq!(pcg.next_u64(), next);
    }
}
