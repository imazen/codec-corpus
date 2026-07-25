//! Deterministic, dependency-free pseudo-random number generator.
//!
//! The corruption generators must be reproducible from `(ref_id, seed, params)`
//! alone — no `rand` crate, no OS entropy, no platform-dependent behavior. This
//! is a plain [SplitMix64](https://prng.di.unimi.it/splitmix64.c) generator: a
//! single `u64` state advanced by a fixed mixing function. Given the same seed
//! it produces the same byte stream on every platform and every run.

/// A minimal SplitMix64 PRNG. Seeded once, advanced deterministically.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Create a generator seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the state and return the next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Return the next value in `0..n` (uniform enough for corpus generation).
    ///
    /// `n` must be non-zero; callers in this module always pass a positive
    /// dimension or count.
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0, "SplitMix64::below requires n > 0");
        self.next_u64() % n
    }

    /// Return the next byte.
    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }

    /// Return a float in `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        // Take the top 24 bits for a uniformly-distributed mantissa.
        let bits = (self.next_u64() >> 40) as u32; // 24 bits
        (bits as f32) / (1u32 << 24) as f32
    }
}

/// Derive a stable per-entry seed from a reference id string and a base seed.
///
/// Uses the same SplitMix64 mixing over the FNV-1a hash of `ref_id` so that
/// two different references with the same base seed produce different (but
/// reproducible) corruptions.
pub fn seed_for(ref_id: &str, base_seed: u64) -> u64 {
    // FNV-1a over the bytes of ref_id.
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for &b in ref_id.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash ^= base_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    // One mix pass so adjacent seeds don't produce adjacent streams.
    let mut g = SplitMix64::new(hash);
    g.next_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_stream() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        // Extremely unlikely to collide on the first draw.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_in_range() {
        let mut g = SplitMix64::new(7);
        for _ in 0..1000 {
            assert!(g.below(10) < 10);
        }
    }

    #[test]
    fn f32_in_unit_interval() {
        let mut g = SplitMix64::new(7);
        for _ in 0..1000 {
            let v = g.next_f32();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn seed_for_stable_and_distinct() {
        assert_eq!(seed_for("photo/01", 0), seed_for("photo/01", 0));
        assert_ne!(seed_for("photo/01", 0), seed_for("photo/02", 0));
        assert_ne!(seed_for("photo/01", 0), seed_for("photo/01", 1));
    }
}
