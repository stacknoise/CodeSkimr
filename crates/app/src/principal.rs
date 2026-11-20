//! Who runs the commands of this process.
//!
//! This is the only place that creates a principal. `codeskimr scan` and the other commands of the
//! command line run on the user's own machine at the user's own terminal, so the caller is the
//! local owner. `codeskimr mcp` and the MCP endpoint answer agents (runs as [`remote_client`], which may not
//! confirm or start an indexer); the commands of the HTTP layer of `codeskimr serve` answer the
//! browser (runs as [`browser_client`], which may start an indexer for a confirmed project but never
//! confirm; ADR 0033, decisions 5 and 13, ADR 0072). A team server resolves the
//! principal of every request from its credentials instead and never calls this.

use codeskimr_domain::{Permission, PermissionSet, Principal};

/// The caller of a command that was started on this machine.
#[must_use]
pub const fn resolve() -> Principal {
    Principal::local_owner()
}

/// The caller of an MCP request (`codeskimr mcp` and the endpoint `/mcp` of `codeskimr serve`):
/// everything the local owner may do except confirming, running or revoking an indexer and asking
/// for the confirmation window. An agent behind MCP never starts a build tool (ADR 0033, decisions
/// 5 and 13; ADR 0072, decision 2).
#[must_use]
pub fn remote_client() -> Principal {
    Principal::limited(remote_rights())
}

/// The caller of a command of the HTTP layer of `codeskimr serve` (the page in the browser, or any
/// holder of the installation token): the rights of [`remote_client`] plus running an indexer,
/// which the scan of the UI needs once the owner has confirmed the project, and taking a
/// confirmation back and asking for the window in which the owner confirms (ADR 0071). It never
/// confirms: [`Permission::ConfirmIndexer`] stays with the person at the terminal (ADR 0072,
/// decision 2).
#[must_use]
pub fn browser_client() -> Principal {
    Principal::limited(
        [
            Permission::RunIndexer,
            Permission::RevokeIndexer,
            Permission::OpenConfirmation,
        ]
        .into_iter()
        .fold(remote_rights(), PermissionSet::with),
    )
}

/// The rights both clients hold.
fn remote_rights() -> PermissionSet {
    [
        Permission::Read,
        Permission::ReadSource,
        Permission::Scan,
        Permission::Annotate,
        Permission::Manage,
        Permission::RegisterProject,
        Permission::Admin,
    ]
    .into_iter()
    .fold(PermissionSet::none(), PermissionSet::with)
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

/// A caller who may annotate but not read, for the tests of the two rights apart.
#[cfg(test)]
#[must_use]
pub fn annotator() -> Principal {
    Principal::limited(PermissionSet::none().with(Permission::Annotate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_mcp_client_may_neither_confirm_nor_run_nor_revoke_nor_open_the_window() {
        let client = remote_client();
        for denied in [
            Permission::ConfirmIndexer,
            Permission::RunIndexer,
            Permission::RevokeIndexer,
            Permission::OpenConfirmation,
        ] {
            assert!(!client.holds(denied), "{denied:?}");
        }
        assert!(client.holds(Permission::Scan));
    }

    #[test]
    fn the_browser_client_may_run_revoke_and_ask_for_the_window_but_never_confirm() {
        let client = browser_client();
        assert!(client.holds(Permission::RunIndexer));
        assert!(client.holds(Permission::RevokeIndexer));
        assert!(client.holds(Permission::OpenConfirmation));
        assert!(!client.holds(Permission::ConfirmIndexer));
    }

    #[test]
    fn the_browser_client_holds_everything_the_mcp_client_holds() {
        for permission in Permission::ALL {
            if remote_client().holds(*permission) {
                assert!(browser_client().holds(*permission), "{permission:?}");
            }
        }
    }

    #[test]
    fn only_the_owner_at_the_terminal_holds_every_right() {
        let held = |principal: Principal| {
            Permission::ALL
                .iter()
                .filter(|permission| principal.holds(**permission))
                .count()
        };
        assert_eq!(held(resolve()), Permission::ALL.len());
        assert_eq!(held(browser_client()), Permission::ALL.len() - 1);
        assert_eq!(held(remote_client()), Permission::ALL.len() - 4);
    }
}
