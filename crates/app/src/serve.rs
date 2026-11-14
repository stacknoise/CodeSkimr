//! `codeskimr serve`: the local server, without opening a browser (blueprint 10.2, 4.3).

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::CredentialStore;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_mcp::HttpService;
use codeskimr_secrets::{OsCredentialStore, OsKeystore};
use codeskimr_server::{CommandApi, EditorSettings, Server, UiOptions, UiSettings};

use crate::consts::URL_SCHEME;
use crate::locations::Locations;
use crate::running::RunningScans;
use crate::wiring::os_secrets;
use crate::{config_files, installation_token, mcp, principal, ui_commands};

/// Serves until the user interrupts the program. `listening` is called once with the address as
/// soon as the server answers.
///
/// The server listens on the loopback address and the port of `server.port`, nothing else
/// (blueprint S-04).
///
/// # Errors
/// - Any coded failure of the configuration.
/// - `SEC-402` or `SEC-601` if the keystore of the installation token cannot be used.
/// - `API-301` if the port is taken, `APP-901` if the runtime fails.
pub fn run(
    global: &GlobalOptions,
    locations: &Locations,
    listening: impl FnOnce(&str),
) -> Result<(), CodedError> {
    let config = config_files::load(locations, global, None)?;
    let store = OsCredentialStore::new(OsKeystore);
    let secrets = os_secrets();
    let events = EventBus::new();
    // The UI and the MCP service refuse a second scan of a project together.
    let running = RunningScans::default();
    let parts = Parts {
        ui: ui_options(&config, locations),
        api: ui_commands::command_api_probing(
            global,
            locations,
            (events.clone(), Arc::clone(&secrets)),
            (None, running.clone()),
        )?,
        mcp: Some(mcp::http_service_sharing(
            &config,
            locations,
            (events.clone(), &secrets),
            running,
        )?),
        events,
        token_lock: Some(locations.token_lock()),
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
        .block_on(serve(
            &store,
            config.server.port,
            parts,
            listening,
            interrupted(),
        ))
}

/// What the browser UI gets from the configuration. Without a built UI folder the server answers
/// the API only.
fn ui_options(config: &codeskimr_application::config::Config, locations: &Locations) -> UiOptions {
    UiOptions {
        dir: locations.existing_dir(&config.ui.dir),
        embedded: crate::embedded_ui::FILES,
        launch_lifetime: Duration::from_secs(u64::from(config.ui.launch_code_seconds)),
        settings: UiSettings {
            editor: EditorSettings {
                url_template: config.editor.url_template.clone(),
                allowed_schemes: config.editor.allowed_schemes.clone(),
            },
            default_theme: config.ui.default_theme.clone(),
        },
    }
}

/// What the server offers: the UI, the commands over REST and the events over the WebSocket. The
/// commands publish on the same bus the WebSocket streams.
pub(crate) struct Parts {
    pub ui: UiOptions,
    pub api: CommandApi,
    pub mcp: Option<HttpService>,
    pub events: EventBus,
    /// File that serialises the first start of several programs (`None`: no lock, for tests).
    pub token_lock: Option<PathBuf>,
}

/// Wires the server and runs it until `shutdown` completes.
pub(crate) async fn serve(
    store: &impl CredentialStore,
    port: u16,
    parts: Parts,
    listening: impl FnOnce(&str),
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), CodedError> {
    let token = match &parts.token_lock {
        Some(lock) => installation_token::load_or_create_locked(store, lock)?,
        None => installation_token::load_or_create(store)?,
    };
    let server = Server::new(
        parts.api,
        parts.events,
        principal::remote_client(),
        token,
        parts.ui,
    );
    let server = match parts.mcp {
        Some(service) => server.with_mcp(service),
        None => server,
    };
    let bound = server
        .bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .await?;
    let mut url = String::from(URL_SCHEME);
    url.push_str(&bound.address()?.to_string());
    ui_commands::mark_ready();
    listening(&url);
    bound.serve(shutdown).await
}

/// A server without any command, for the tests of the wiring.
#[cfg(test)]
pub(crate) fn bare_parts(ui: UiOptions) -> Parts {
    Parts {
        ui,
        mcp: None,
        api: CommandApi::new(std::sync::Arc::new(
            codeskimr_application::registry::Registry::new(),
        )),
        events: EventBus::new(),
        token_lock: None,
    }
}

/// Completes when the user presses Ctrl+C. If the signal cannot be watched, the server runs on
/// until the process is ended another way.
async fn interrupted() {
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
}

/// The UI options of the shipped defaults, without a UI folder, for the tests of the wiring.
#[cfg(test)]
pub(crate) fn test_ui() -> UiOptions {
    let config = codeskimr_application::config::Config::defaults().unwrap();
    let locations = Locations {
        data_dir: std::path::PathBuf::new(),
        languages_dir: std::path::PathBuf::new(),
        user_config: None,
        bases: Vec::new(),
    };
    ui_options(&config, &locations)
}

/// Runs the server of a test on a free port in a runtime of its own until `stopped` completes.
/// The address is sent to `found` as soon as the server answers.
#[cfg(test)]
pub(crate) fn serve_for_test(
    store: &impl CredentialStore,
    parts: Parts,
    found: &std::sync::mpsc::Sender<String>,
    stopped: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), CodedError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    let listening = |url: &str| found.send(url.to_owned()).unwrap();
    let shutdown = async {
        stopped.await.ok();
    };
    runtime.block_on(serve(store, 0, parts, listening, shutdown))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    use codeskimr_secrets::KeystoreCredentials;
    use tokio::sync::oneshot::channel;

    use super::*;
    use crate::memory_keystore::Memory;

    fn get(url: &str, path: &str, host: &str) -> String {
        let address = url.trim_start_matches(URL_SCHEME);
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn the_server_listens_on_loopback_answers_and_stops() {
        let store = KeystoreCredentials::new(Memory::default());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (stop, stopped) = channel::<()>();
        let (found, url) = channel::<String>();
        let client = std::thread::spawn(move || {
            let url = runtime_free_wait(url);
            assert!(url.starts_with("http://127.0.0.1:"), "{url}");
            let own = url.trim_start_matches(URL_SCHEME).to_owned();
            let list = get(&url, "/api/v1/commands", &own);
            assert!(list.starts_with("HTTP/1.1 200"), "{list}");
            assert!(list.ends_with("[]"), "{list}");
            let foreign = get(&url, "/api/v1/commands", "evil.example");
            assert!(foreign.starts_with("HTTP/1.1 403"), "{foreign}");
            stop.send(()).unwrap();
        });
        runtime
            .block_on(serve(
                &store,
                0,
                bare_parts(test_ui()),
                |url| found.send(url.to_owned()).unwrap(),
                async {
                    stopped.await.ok();
                },
            ))
            .unwrap();
        client.join().unwrap();
    }

    fn runtime_free_wait(url: tokio::sync::oneshot::Receiver<String>) -> String {
        url.blocking_recv().unwrap()
    }
}
