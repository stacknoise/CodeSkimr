//! Local web server: REST, WebSocket and the embedded UI.
//!
//! The server is a driving adapter (blueprint 4.1). It holds no logic of its own: every route
//! calls a command through the registry, which checks the permission once (blueprint 10.1), and
//! every failure leaves as a registered code (blueprint 11).
//!
//! In single-user mode it listens on the loopback address only and checks `Host` and `Origin`
//! of every request; a request that changes something also needs the installation token. CORS is
//! closed, no header of any kind opens it (blueprint S-04). It also serves the built UI with a
//! strict content security policy and gives the browser the token through a one-time launch code
//! (ADR 0028, ADR 0030). See `docs/server.md`.

mod assets;
mod commands;
mod consts;
mod error;
mod events;
mod guard;
mod launch;
mod listen;
mod routes;
mod server;
mod session;
mod settings;
mod state;
mod token;
mod ui;

pub use assets::EmbeddedFile;
pub use commands::{CommandApi, CommandInfo};
pub use consts::TOKEN_PREFIX;
pub use error::{ErrorBody, status_of};
pub use events::WireEvent;
pub use server::{BoundServer, Server, ensure_loopback};
pub use settings::{EditorSettings, UiOptions, UiSettings};
pub use token::InstallationToken;
