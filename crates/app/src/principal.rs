//! Who runs the commands of this process.
//!
//! This is the only place that creates a principal. `codeskimr scan` and the other commands of the
//! command line run on the user's own machine at the user's own terminal, so the caller is the
//! local owner. `codeskimr mcp` and `codeskimr serve` answer clients that are not at the terminal
//! (an agent, a browser, another machine), so they run as [`remote_client`], which may not give
//! consent to an indexer or start one (ADR 0033, decisions 5 and 13). A team server resolves the
//! principal of every request from its credentials instead and never calls this.

use codeskimr_domain::{Permission, PermissionSet, Principal};

/// The caller of a command that was started on this machine.
#[must_use]
pub const fn resolve() -> Principal {
    Principal::local_owner()
}

/// The caller of an MCP or HTTP request: everything the local owner may do except confirming or
/// running an indexer. Those two rights stay with the person at the terminal.
#[must_use]
pub fn remote_client() -> Principal {
    let rights = [
        Permission::Read,
        Permission::ReadSource,
        Permission::Scan,
        Permission::Manage,
        Permission::RegisterProject,
        Permission::Admin,
    ];
    Principal::limited(
        rights
            .into_iter()
            .fold(PermissionSet::none(), PermissionSet::with),
    )
}

/// A caller without any right, for the tests that check a refusal.
#[cfg(test)]
#[must_use]
pub fn nobody() -> Principal {
    Principal::limited(PermissionSet::none())
}

/// A caller who may read the index but not the source text, for the tests of that boundary.
#[cfg(test)]
#[must_use]
pub fn reader() -> Principal {
    Principal::limited(PermissionSet::none().with(Permission::Read))
}
