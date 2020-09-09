/*! This is an implementation of a PRNG from the PCG family.
 *  Specifically, it implements PCG-XSH-RS-64/32 (MCG).
 *  For more information on the PCG family of PRNGs,
 *  see https://www.pcg-random.org/paper.html
 *
 *  Although the specific algorithm implemented is
 *  reasonably secure, this implementation has not been
 *  thoroughly tested and should be assumed not to be secure.
 *  It is not suitable for secure applications.
 *  Use at your own peril.
 *
 *  # Example use
 *  ```
 *  let seed: u64 = 12345; // or any u64 seed, to taste
 *  let mut pcg = Pcg::seed_from_u64(seed);
 *  
 *  let x = pcg.next_u32();
 *
 *  let mut other_pcg = pcg.new_stream();
 *  let y = other_pcg.next_u32();
 *
 *  assert_ne!(x, y);
 *  ```
 */
/// 8^20 + 3, an arbitrary number that provides an acceptable period
const MULTIPLIER: u64 = 0x1000000000000003;
/// the inverse of MULTIPLIER; (MULTIPLIER*INVERSE)%(2^64) = 1
const INVERSE: u64 = 0x1AAAAAAAAAAAAAAB;
const BYTE_LEN: usize = 8;

#[derive(Default)]
pub struct PcgSeed(pub [u8; BYTE_LEN]);

use rand_core::*;
use std::num::Wrapping;

#[derive(Clone)]
pub struct Pcg {
    state: u64,
}

impl Pcg {
    #[cfg(test)]
    pub fn get_state(&self) -> u64 {
        self.state
    }

    /// Advances the state by n steps, as if calling next_u32() n times
    pub fn skip(&mut self, n: i32) {
        if n == 0 {
            return;
        }
        let mut state = Wrapping(self.state);
        if n > 0 {
            for _ in 0..n {
                state *= Wrapping(MULTIPLIER);
            }
        } else {
            for _ in n..0 {
                state *= Wrapping(INVERSE);
            }
        }
        self.state = state.0;
    }

    /// Creates a new Pcg instance with a unique state seeded from the
    /// output of this Pcg instance.
    pub fn new_stream(&mut self) -> Pcg {
        Self::seed_from_u64(self.next_u64())
    }
}

impl RngCore for Pcg {
    /// Generate a random u32, advancing the state one step.
    fn next_u32(&mut self) -> u32 {
        self.state = (Wrapping(self.state) * Wrapping(MULTIPLIER)).0;
        ((self.state ^ (self.state >> 22)) >> (22 + (self.state >> 61))) as u32
    }

    /// Generate a random u64. Note that this advances the state
    /// two steps, as each step only provides 32 bits of output.
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) ^ (self.next_u32() as u64)
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
        Self {
            state: if seed == 0 { 1 } else { seed }, // must not have zero as state
        }
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
        let mut state = (Wrapping(seed) * Wrapping(MULTIPLIER)).0;
        let mut next: u64 = (state ^ (state >> 22)) >> (22 + (state >> 61)) << 32;
        state = (Wrapping(state) * Wrapping(MULTIPLIER)).0;
        next ^= ((state ^ (state >> 22)) >> (22 + (state >> 61))) & 0xFFFFFFFF;

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
        let mut next_eight_expected_bytes = [0; 8];
        for i in 0..4 {
            next_eight_expected_bytes[i + 4] = ((next >> 8 * i) % 256) as u8;
        }
        for i in 0..4 {
            next_eight_expected_bytes[i] = ((secondnext >> 8 * i) % 256) as u8;
        }

        let mut arr = [0; 8];
        let mut pcg = Pcg::seed_from_u64(seed);
        pcg.fill_bytes(&mut arr);
        assert_eq!(arr, next_eight_expected_bytes);

        pcg = Pcg::seed_from_u64(seed);
        assert!(pcg.try_fill_bytes(&mut arr).is_ok());
        assert_eq!(arr, next_eight_expected_bytes);
    }

    #[test]
    fn test_skip() {
        let seed = rand::random::<u64>();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER) * Wrapping(MULTIPLIER)).0;
        let next = ((state ^ (state >> 22)) >> (22 + (state >> 61))) as u32;

        let mut pcg = Pcg::seed_from_u64(seed);
        pcg.skip(1);
        assert_eq!(pcg.next_u32(), next);
    }

    #[test]
    fn test_skip_backwards() {
        let seed = rand::random::<u64>();
        let skips = rand::random::<i8>();
        let mut pcg = Pcg::seed_from_u64(seed);
        pcg.skip(skips as i32);
        pcg.skip(-skips as i32);
        assert_eq!(pcg.get_state(), seed);
    }

    #[test]
    fn test_no_zeroes_in_state() {
        let mut pcg = Pcg::seed_from_u64(0);
        assert_ne!(pcg.get_state(), 0);

        for _ in 0..100 {
            pcg.skip(1);
            assert_ne!(pcg.get_state(), 0);
        }
    }

    #[test]
    fn test_clone() {
        let mut parent = Pcg::seed_from_u64(rand::random::<u64>());
        let mut child = parent.clone();
        assert_eq!(child.next_u64(), parent.next_u64());
        parent.skip(1);
        assert_ne!(child.next_u64(), parent.next_u64());
    }

    #[test]
    fn test_new_stream() {
        let mut parent = Pcg::seed_from_u64(rand::random::<u64>());
        let mut child = parent.new_stream();

        parent.skip(-2);
        let seed = parent.next_u64();
        let state = (Wrapping(seed) * Wrapping(MULTIPLIER)).0;
        let next = ((state ^ (state >> 22)) >> (22 + (state >> 61))) as u32;
        assert_eq!(child.next_u32(), next);
    }
}
