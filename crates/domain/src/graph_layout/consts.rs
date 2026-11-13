//! Technical constants of the graph layout.

/// Divisor that turns a width into a half width and a sum into a midpoint.
pub const HALF: i64 = 2;
/// Median keys are stored doubled, so that the midpoint of two neighbours stays an integer.
pub const KEY_SCALE: usize = 2;
/// Sweeps alternate between downward (even) and upward (odd).
pub const SWEEP_CYCLE: u32 = 2;
/// Placement passes alternate between downward (even) and upward (odd).
pub const PASS_CYCLE: u32 = 2;
