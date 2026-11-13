//! Local web server: REST, WebSocket and the embedded UI.
//!
//! The server is a driving adapter (blueprint 4.1). It holds no logic of its own: every route
//! calls a command through the registry, which checks the permission once (blueprint 10.1), and
//! every failure leaves as a registered code (blueprint 11).
//!
//! In single-user mode it listens on the loopback address only and checks `Host` and `Origin`
//! of every request; a request that changes something also needs the installation token. CORS is
//! closed, no header of any kind opens it (blueprint S-04). See `docs/server.md`.

mod commands;
mod consts;
mod error;
mod events;
mod guard;
mod routes;
mod server;
mod state;
mod token;

pub use commands::{CommandApi, CommandInfo};
pub use consts::TOKEN_PREFIX;
pub use error::{ErrorBody, status_of};
pub use events::WireEvent;
pub use server::{BoundServer, Server, ensure_loopback};
pub use token::InstallationToken;
