//! Fixed values of the resolver.

/// The limit of candidates of a place through the bare name when nothing configures it
/// (`scan.max_candidates` in `config/default.toml`).
pub const DEFAULT_MAX_CANDIDATES: usize = 100;

/// A character that can be part of a name besides letters and digits.
pub const NAME_JOINER: char = '_';
