//! A cheap hasher for maps and sets keyed by numbers (ids and positions). The standard hasher is
//! built to resist attacks on maps that take foreign keys; these keys are made by the scan, and
//! the coupling of a snapshot looks them up once per reference.

use std::hash::{BuildHasherDefault, Hasher};

use crate::consts::{ID_HASH_FINISH_ROTATION, ID_HASH_MULTIPLIER, ID_HASH_ROTATION};

/// Mixes each written number into the state with a rotation, an exclusive or and a multiplication.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct IdHasher(u64);

impl IdHasher {
    fn mix(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(ID_HASH_ROTATION) ^ value).wrapping_mul(ID_HASH_MULTIPLIER);
    }
}

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0.rotate_left(ID_HASH_FINISH_ROTATION)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.mix(u64::from(*byte));
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.mix(value);
    }

    fn write_usize(&mut self, value: usize) {
        self.mix(u64::try_from(value).unwrap_or(u64::MAX));
    }
}

/// The hasher of maps and sets keyed by ids.
pub(crate) type IdHash = BuildHasherDefault<IdHasher>;
