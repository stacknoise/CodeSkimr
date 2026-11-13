//! Starting the server: the address check of S-04, the router and the serve loop.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Principal;
use tokio::net::TcpListener;

use crate::assets::Assets;
use crate::commands::CommandApi;
use crate::consts::{
    COMMAND_PATH, COMMANDS_PATH, EVENTS_PATH, EXCHANGE_PATH, LAUNCH_CODE_PATH, UI_CONFIG_PATH,
};
use crate::guard::{Guard, screen};
use crate::launch::LaunchCodes;
use crate::state::AppState;
use crate::token::InstallationToken;
use crate::ui::{self, UiOptions};
use crate::{events, routes, session};

/// The local server before it listens.
pub struct Server {
    api: CommandApi,
    events: EventBus,
    principal: Principal,
    token: InstallationToken,
    ui: UiOptions,
}

/// A server that listens and has not started to answer yet.
pub struct BoundServer {
    listener: TcpListener,
    router: Router,
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
        }
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
        let state = AppState {
            api: Arc::new(self.api),
            guard: Arc::clone(&guard),
            principal: self.principal,
            events: self.events,
            launch: Arc::new(LaunchCodes::new(self.ui.launch_lifetime)),
            ui: Arc::new(self.ui.settings),
            assets: self.ui.dir.map(|dir| Arc::new(Assets::new(dir))),
        };
        let router = Router::new()
            .route(COMMANDS_PATH, get(routes::list))
            .route(COMMAND_PATH, post(routes::run))
            .route(EVENTS_PATH, get(events::stream))
            .route(UI_CONFIG_PATH, get(ui::config))
            .route(LAUNCH_CODE_PATH, post(session::issue).get(session::state))
            .route(EXCHANGE_PATH, post(session::exchange))
            .fallback(routes::not_found)
            .method_not_allowed_fallback(routes::wrong_method)
            .layer(from_fn_with_state(guard, screen))
            .with_state(state);
        Ok(BoundServer { listener, router })
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
    /// `APP-901` if the serve loop fails.
    pub async fn serve(
        self,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), CodedError> {
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))
    }
}
