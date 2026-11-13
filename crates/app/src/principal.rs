//! Who runs the commands of this process.
//!
//! This is the only place that creates a principal. `codeskimr scan` and `codeskimr mcp` run on
//! the user's own machine without login, so the caller is the local owner. A team server resolves
//! the principal of every request from its credentials instead and never calls this.

use codeskimr_domain::Principal;

/// The caller of a command that was started on this machine.
#[must_use]
pub const fn resolve() -> Principal {
    Principal::local_owner()
}
