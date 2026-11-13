//! Syntax fingerprints: winnowing over normalised token streams.
//!
//! One implementation serves `find_similar_code` and the duplicate check of `check_conventions`
//! (blueprint 10.4). A parser adapter turns a function body into a stream of [`TokenId`]s in which
//! identifiers and literals are abstracted; [`winnow`] selects a small, position-stable subset of
//! the hashes of all token n-grams. Two bodies that share a run of at least
//! `window_size + ngram_size - 1` tokens are guaranteed to share a fingerprint.
//!
//! The hash is implemented here on purpose: the domain may depend on nothing but `codes`, std and
//! `thiserror` (blueprint 4.2), and the values must stay stable across releases because storage
//! keeps them.

use std::collections::{HashSet, VecDeque};
use std::num::NonZeroUsize;

use crate::ModelError;
use crate::consts::{
    FNV_OFFSET_BASIS, FNV_PRIME, MIX_MULTIPLIER_FIRST, MIX_MULTIPLIER_SECOND, MIX_SHIFT_FIRST,
    MIX_SHIFT_LAST, MIX_SHIFT_SECOND, ROLLING_BASE, SIMILARITY_NONE, TOKEN_IDENTIFIER,
    TOKEN_LITERAL,
};

/// One normalised syntax token, identified by a hash of its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenId(u64);

impl TokenId {
    /// The token of a syntax node kind such as `if_expression` or a keyword.
    #[must_use]
    pub fn node_kind(kind: &str) -> Self {
        Self(kind.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        }))
    }

    /// The token that stands for every identifier.
    #[must_use]
    pub fn identifier() -> Self {
        Self::node_kind(TOKEN_IDENTIFIER)
    }

    /// The token that stands for every literal.
    #[must_use]
    pub fn literal() -> Self {
        Self::node_kind(TOKEN_LITERAL)
    }
}

/// How many tokens form an n-gram and how many n-grams a winnowing window spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FingerprintParams {
    ngram_size: NonZeroUsize,
    window_size: NonZeroUsize,
}

impl FingerprintParams {
    /// Creates the parameters.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::NotPositive`] if either value is zero.
    pub fn new(ngram_size: usize, window_size: usize) -> Result<Self, ModelError> {
        Ok(Self {
            ngram_size: NonZeroUsize::new(ngram_size).ok_or(ModelError::NotPositive)?,
            window_size: NonZeroUsize::new(window_size).ok_or(ModelError::NotPositive)?,
        })
    }

    /// Number of tokens per n-gram.
    #[must_use]
    pub const fn ngram_size(&self) -> usize {
        self.ngram_size.get()
    }

    /// Number of consecutive n-grams per window.
    #[must_use]
    pub const fn window_size(&self) -> usize {
        self.window_size.get()
    }
}

/// A selected n-gram hash and the index of its first token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint {
    /// Hash of the n-gram.
    pub hash: u64,
    /// Index of the first token of the n-gram in the stream.
    pub position: u32,
}

/// The fingerprints of one body, ordered by position.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fingerprints(Vec<Fingerprint>);

impl Fingerprints {
    /// Wraps stored entries, for example read back from the database.
    #[must_use]
    pub const fn from_entries(entries: Vec<Fingerprint>) -> Self {
        Self(entries)
    }

    /// The entries in order of position.
    #[must_use]
    pub fn entries(&self) -> &[Fingerprint] {
        &self.0
    }

    /// Whether no fingerprint was selected, for example because the body is too short.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn hashes(&self) -> HashSet<u64> {
        self.0.iter().map(|entry| entry.hash).collect()
    }

    /// Jaccard similarity of the distinct hashes, from 0.0 (nothing shared) to 1.0 (identical).
    /// Two empty sets share nothing.
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f64 {
        let (mine, theirs) = (self.hashes(), other.hashes());
        ratio(
            mine.intersection(&theirs).count(),
            mine.union(&theirs).count(),
        )
    }

    /// Share of the distinct hashes of `self` that `other` contains, from 0.0 to 1.0. It answers
    /// "is this snippet part of that function".
    #[must_use]
    pub fn containment(&self, other: &Self) -> f64 {
        let (mine, theirs) = (self.hashes(), other.hashes());
        ratio(mine.intersection(&theirs).count(), mine.len())
    }
}

fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        return SIMILARITY_NONE;
    }
    f64::from(saturate(part)) / f64::from(saturate(whole))
}

fn saturate(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// Selects the fingerprints of `tokens`: of every window of `window_size` consecutive n-gram
/// hashes the smallest one, the rightmost on ties, each position once.
///
/// A stream shorter than one n-gram yields nothing; one with fewer n-grams than a window is
/// treated as a single window.
#[must_use]
pub fn winnow(tokens: &[TokenId], params: FingerprintParams) -> Fingerprints {
    let grams = ngram_hashes(tokens, params.ngram_size());
    let window_size = params.window_size();
    let first_full = window_size.min(grams.len());
    let mut window: VecDeque<(usize, u64)> = VecDeque::new();
    let mut selected: Vec<Fingerprint> = Vec::new();
    for (index, hash) in grams.iter().copied().enumerate() {
        while window.back().is_some_and(|&(_, last)| last >= hash) {
            window.pop_back();
        }
        window.push_back((index, hash));
        while window
            .front()
            .is_some_and(|&(start, _)| start + window_size <= index)
        {
            window.pop_front();
        }
        if index + 1 >= first_full {
            keep_minimum(&window, &mut selected);
        }
    }
    Fingerprints(selected)
}

fn keep_minimum(window: &VecDeque<(usize, u64)>, selected: &mut Vec<Fingerprint>) {
    let Some(&(position, hash)) = window.front() else {
        return;
    };
    let position = saturate(position);
    if selected.last().is_none_or(|last| last.position != position) {
        selected.push(Fingerprint { hash, position });
    }
}

/// Hashes of all n-grams, computed with a rolling polynomial hash and mixed afterwards.
fn ngram_hashes(tokens: &[TokenId], size: usize) -> Vec<u64> {
    if tokens.len() < size {
        return Vec::new();
    }
    let weight = (1..size).fold(1_u64, |power, _| power.wrapping_mul(ROLLING_BASE));
    let extend = |hash: u64, token: &TokenId| hash.wrapping_mul(ROLLING_BASE).wrapping_add(token.0);
    let mut hash = tokens.iter().take(size).fold(0_u64, extend);
    let mut hashes = vec![mix(hash)];
    for (outgoing, incoming) in tokens.iter().zip(tokens.iter().skip(size)) {
        hash = extend(hash.wrapping_sub(outgoing.0.wrapping_mul(weight)), incoming);
        hashes.push(mix(hash));
    }
    hashes
}

/// Spreads the bits so that "smallest hash" does not favour particular token kinds.
const fn mix(value: u64) -> u64 {
    let step = (value ^ (value >> MIX_SHIFT_FIRST)).wrapping_mul(MIX_MULTIPLIER_FIRST);
    let step = (step ^ (step >> MIX_SHIFT_SECOND)).wrapping_mul(MIX_MULTIPLIER_SECOND);
    step ^ (step >> MIX_SHIFT_LAST)
}
