//! Deterministic pseudo-random numbers, so that every benchmark run measures the same data.

use crate::consts::{LCG_INCREMENT, LCG_MULTIPLIER, LCG_SHIFT};

/// A linear congruential generator with a fixed seed.
#[derive(Debug, Clone, Copy)]
pub struct Lcg(u64);

impl Lcg {
    /// Starts the sequence of `seed`.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next number in `0..limit`; `limit` must not be zero.
    pub fn below(&mut self, limit: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        usize::try_from(self.0 >> LCG_SHIFT).unwrap_or_default() % limit
    }
}
