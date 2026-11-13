//! What every route handler shares.

use std::sync::Arc;

use codeskimr_application::events::EventBus;
use codeskimr_domain::Principal;

use crate::assets::Assets;
use crate::commands::CommandApi;
use crate::guard::Guard;
use crate::launch::LaunchCodes;
use crate::ui::UiSettings;

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
    /// The one-time launch code of the browser (ADR 0028).
    pub launch: Arc<LaunchCodes>,
    /// The settings the UI reads before it starts.
    pub ui: Arc<UiSettings>,
    /// The built UI, if the server has one.
    pub assets: Option<Arc<Assets>>,
}
