//! What every route handler shares.

use std::sync::Arc;

use codeskimr_application::events::EventBus;
use codeskimr_domain::Principal;

use crate::commands::CommandApi;
use crate::guard::Guard;

/// The shared state of the router. Cloning is cheap: everything is behind an `Arc` or `Copy`.
#[derive(Clone)]
pub struct AppState {
    /// The commands reachable over REST.
    pub api: Arc<CommandApi>,
    /// The checks for `Host`, `Origin` and the token.
    pub guard: Arc<Guard>,
    /// The caller, resolved once by the composition root (blueprint 4.3).
    pub principal: Principal,
    /// Where the events of the application come from.
    pub events: EventBus,
}
