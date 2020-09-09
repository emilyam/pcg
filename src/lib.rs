// 8^20 + 3, an arbitrary number that provides an acceptable cycle length
const MULTIPLIER: u64 = 1152921504606846979;
const BYTE_LEN: usize = 8;

#[derive(Debug)]
#[derive(Default)]
pub struct PcgSeed(pub [u8; BYTE_LEN]);

use rand_core::*;

pub struct Pcg {
    state: u64,
}

impl Pcg {
    #[cfg(test)]
    pub fn get_state(&self) -> u64 {
        self.state
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
        seed ^= (mutarr[i] as u64) << 8*i;
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
}
