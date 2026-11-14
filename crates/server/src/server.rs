//! Starting the server: the address check of S-04, the router and the serve loop.

use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::from_fn_with_state;
use axum::routing::{MethodRouter, any_service, get, post};
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Principal;
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, watch};

use crate::assets::Assets;
use crate::commands::CommandApi;
use crate::consts::{
    COMMAND_PATH, COMMANDS_PATH, EVENTS_PATH, EXCHANGE_PATH, LAUNCH_CODE_PATH, MAX_EVENT_STREAMS,
    MCP_PATH, UI_CONFIG_PATH,
};
use crate::guard::{Guard, require_token, screen};
use crate::launch::LaunchCodes;
use crate::listen;
use crate::settings::UiOptions;
use crate::state::AppState;
use crate::token::InstallationToken;
use crate::ui;
use crate::{events, routes, session};

/// The local server before it listens.
pub struct Server {
    api: CommandApi,
    events: EventBus,
    principal: Principal,
    token: InstallationToken,
    ui: UiOptions,
    mcp: Option<MethodRouter<AppState>>,
}

/// A server that listens and has not started to answer yet.
pub struct BoundServer {
    listener: TcpListener,
    router: Router,
    stop: watch::Sender<()>,
}

/// Lets `address` pass if it is a loopback address (blueprint S-04).
///
/// In single-user mode there is no other choice: a server on another address would give the
/// local owner's rights to anyone on the network. Remote access belongs to the team server's
/// server mode with TLS or a trusted proxy (ADR 0019) and does not exist yet.
///
/// # Errors
/// `API-102` for any other address, including `0.0.0.0` and `::`.
pub fn ensure_loopback(address: SocketAddr) -> Result<(), CodedError> {
    if address.ip().is_loopback() {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::BindAddressNotLocal))
    }
}

/// The routes of the server with the layers every request passes. The MCP route, if there is
/// one, is checked for the token on top of that.
fn build_router(state: AppState, guard: Arc<Guard>, mcp: Option<MethodRouter<AppState>>) -> Router {
    let mut router = Router::new()
        .route(COMMANDS_PATH, get(routes::list))
        .route(COMMAND_PATH, post(routes::run))
        .route(EVENTS_PATH, get(events::stream))
        .route(UI_CONFIG_PATH, get(ui::config))
        .route(LAUNCH_CODE_PATH, post(session::issue).get(session::state))
        .route(EXCHANGE_PATH, post(session::exchange))
        .fallback(routes::not_found)
        .method_not_allowed_fallback(routes::wrong_method);
    if let Some(mcp) = mcp {
        let checked = mcp.layer(from_fn_with_state(Arc::clone(&guard), require_token));
        router = router.route(MCP_PATH, checked);
    }
    router
        .layer(from_fn_with_state(guard, screen))
        .with_state(state)
}

impl Server {
    /// A server that offers `api`, streams the events of `events`, and runs every request as
    /// `principal`. Changes need `token`.
    #[must_use]
    pub fn new(
        api: CommandApi,
        events: EventBus,
        principal: Principal,
        token: InstallationToken,
        ui: UiOptions,
    ) -> Self {
        Self {
            api,
            events,
            principal,
            token,
            ui,
            mcp: None,
        }
    }

    /// Offers MCP at `/mcp` through `service`, on the same listener, with the same `Host` and
    /// `Origin` checks as every route. Every request to it needs the installation token, also a
    /// read: MCP answers carry code structure, and no other program on this machine may read it
    /// through the open loopback port (blueprint S-04).
    #[must_use]
    pub fn with_mcp<S>(mut self, service: S) -> Self
    where
        S: tower_service::Service<Request<Body>, Error = Infallible>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse + 'static,
        S::Future: Send + 'static,
    {
        self.mcp = Some(any_service(service));
        self
    }

    /// Listens on `address`; with port 0 the system picks a free port, see
    /// [`BoundServer::address`].
    ///
    /// # Errors
    /// `API-102` if `address` is not a loopback address (checked first, nothing is opened then),
    /// `API-301` if the address cannot be bound.
    pub async fn bind(self, address: SocketAddr) -> Result<BoundServer, CodedError> {
        ensure_loopback(address)?;
        let bind_failed = |_| CodedError::new(ErrorCode::ServerBindFailed);
        let listener = TcpListener::bind(address).await.map_err(bind_failed)?;
        let bound = listener.local_addr().map_err(bind_failed)?;
        let guard = Arc::new(Guard::new(bound, self.token));
        let (stop, stopped) = watch::channel(());
        let state = AppState {
            stop: stopped,
            api: Arc::new(self.api),
            guard: Arc::clone(&guard),
            principal: self.principal,
            events: self.events,
            streams: Arc::new(Semaphore::new(MAX_EVENT_STREAMS)),
            launch: Arc::new(LaunchCodes::new(self.ui.launch_lifetime)),
            ui: Arc::new(self.ui.settings),
            assets: assets_of(self.ui.dir, self.ui.embedded),
        };
        let router = build_router(state, guard, self.mcp);
        Ok(BoundServer {
            listener,
            router,
            stop,
        })
    }
}

impl BoundServer {
    /// The address the server listens on, with the real port.
    ///
    /// # Errors
    /// `API-301` if the system cannot tell.
    pub fn address(&self) -> Result<SocketAddr, CodedError> {
        self.listener
            .local_addr()
            .map_err(|_| CodedError::new(ErrorCode::ServerBindFailed))
    }

    /// Answers requests until `shutdown` completes, then lets running requests finish.
    ///
    /// # Errors
    /// None today; the result stays so that a failing serve loop can be reported.
    pub async fn serve(
        self,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), CodedError> {
        listen::run(self.listener, self.router, self.stop, shutdown).await;
        Ok(())
    }
}

/// The UI files: the folder if there is one, else the files in the program, else none.
fn assets_of(
    dir: Option<std::path::PathBuf>,
    embedded: &'static [crate::assets::EmbeddedFile],
) -> Option<Arc<Assets>> {
    match dir {
        Some(dir) => Some(Arc::new(Assets::new(dir))),
        None if embedded.is_empty() => None,
        None => Some(Arc::new(Assets::embedded(embedded))),
    }
}
