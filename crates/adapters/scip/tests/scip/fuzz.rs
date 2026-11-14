//! Random and damaged input never panics; it is an index or a coded error.

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_scip::decode_index;

use crate::common::{LIMIT, sample_index};

/// A small deterministic generator, so a failure is reproducible.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).expect("fits")
    }
}

fn check(bytes: &[u8]) {
    if let Err(error) = decode_index(bytes, LIMIT) {
        assert!(matches!(
            error.code(),
            ErrorCode::ScipIndexInvalid | ErrorCode::ScipIndexTooLarge
        ));
    }
}

#[test]
fn random_bytes_never_panic() {
    let mut random = Xorshift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..20_000 {
        let length = random.below(96);
        let bytes: Vec<u8> = (0..length)
            .map(|_| random.next().to_le_bytes()[0])
            .collect();
        check(&bytes);
    }
}

#[test]
fn damaged_indexes_never_panic() {
    let valid = sample_index();
    let mut random = Xorshift(0x0123_4567_89ab_cdef);
    for _ in 0..20_000 {
        let mut bytes = valid.clone();
        for _ in 0..=random.below(4) {
            let at = random.below(bytes.len());
            if let Some(byte) = bytes.get_mut(at) {
                *byte = random.next().to_le_bytes()[0];
            }
        }
        if random.below(4) == 0 {
            bytes.truncate(random.below(valid.len()));
        }
        check(&bytes);
    }
}
