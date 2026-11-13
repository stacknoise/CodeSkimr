//! `codeskimr open`: gives the browser the one-time launch code and starts it (ADR 0028).
//!
//! The command reads the installation token from the credential store, so only the user who owns
//! the keystore can do this. It asks the running server for a code, writes the redirect page into
//! a private file, opens the file in the browser and waits until the code has been taken or has
//! expired. Then the file goes.

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AuthError, CredentialStore};
use codeskimr_cli::{GlobalOptions, OpenArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::CredentialId;
use codeskimr_secrets::{OsCredentialStore, OsKeystore};
use secrecy::ExposeSecret;
use serde_json::Value;
use zeroize::Zeroizing;

use crate::consts::{
    APP_DIR, CODE_POLL_MS, ENV_XDG_RUNTIME_DIR, FIELD_CODE, FIELD_PENDING, LAUNCH_CODE_CHARS,
    LAUNCH_CODE_PATH, LAUNCH_DIR, LAUNCH_FRAGMENT, LOOPBACK_HOST, METHOD_GET, METHOD_POST,
    PORT_SEPARATOR, TOKEN_CREDENTIAL_ID, URL_SCHEME,
};
use crate::launch_file::LaunchFile;
use crate::local_http::{Call, call};
use crate::locations::{Locations, Lookup};
use crate::{browser, config_files};

/// What `open` tells the user once the launch file exists.
pub struct Ready<'a> {
    /// Where the file is.
    pub path: &'a Path,
    /// How long the code in it is valid, in seconds.
    pub seconds: u64,
}

/// Runs the command. `ready` is called once, when the file exists; the call returns when the code
/// has been taken or is no longer valid.
///
/// # Errors
/// - Any coded failure of the configuration.
/// - `SEC-402` or `SEC-601` if the keystore cannot be used.
/// - `API-601` if the server is not running (or no token exists yet, which means it never ran).
/// - `APP-202` if the launch file cannot be created, `APP-601` if no browser can be started.
pub fn run(
    args: OpenArgs,
    global: &GlobalOptions,
    locations: &Locations,
    env: Lookup<'_>,
    ready: impl FnOnce(&Ready<'_>),
) -> Result<(), CodedError> {
    let config = config_files::load(locations, global, None)?;
    let plan = Plan {
        port: config.server.port,
        lifetime: Duration::from_secs(u64::from(config.ui.launch_code_seconds)),
        dir: launch_dir(locations, env),
        browser: !args.no_browser,
    };
    launch(&OsCredentialStore::new(OsKeystore), &plan, ready)
}

/// What `launch` needs besides the credential store.
pub(crate) struct Plan {
    pub(crate) port: u16,
    pub(crate) lifetime: Duration,
    pub(crate) dir: PathBuf,
    pub(crate) browser: bool,
}

pub(crate) fn launch(
    store: &impl CredentialStore,
    plan: &Plan,
    ready: impl FnOnce(&Ready<'_>),
) -> Result<(), CodedError> {
    let token = read_token(store)?;
    let code = request_code(plan.port, &token)?;
    let url = [
        URL_SCHEME,
        LOOPBACK_HOST,
        PORT_SEPARATOR,
        &plan.port.to_string(),
        LAUNCH_FRAGMENT,
        code.as_str(),
    ]
    .concat();
    let file = LaunchFile::create(&plan.dir, &url, plan.lifetime)?;
    drop(code);
    ready(&Ready {
        path: file.path(),
        seconds: plan.lifetime.as_secs(),
    });
    if plan.browser {
        browser::open(file.path())?;
    }
    wait_until_taken(plan.port, &token, plan.lifetime);
    Ok(())
}

/// The installation token, or `API-601` if there is none: the server creates it at its first
/// start, so a missing token means that it has never run.
fn read_token(store: &impl CredentialStore) -> Result<Zeroizing<String>, CodedError> {
    let id = CredentialId::new(TOKEN_CREDENTIAL_ID)
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    match store.with_credential(&id, |credential| {
        Ok(Zeroizing::new(
            credential.secret().expose_secret().to_owned(),
        ))
    }) {
        Err(AuthError::NotFound) => Err(CodedError::new(ErrorCode::ServerNotReachable)),
        other => Ok(other?),
    }
}

/// A new launch code from the running server.
fn request_code(port: u16, token: &str) -> Result<Zeroizing<String>, CodedError> {
    let answer = call(&Call {
        port,
        method: METHOD_POST,
        path: LAUNCH_CODE_PATH,
        token,
    })?;
    answer
        .get(FIELD_CODE)
        .and_then(Value::as_str)
        .filter(|code| is_launch_code(code))
        .map(|code| Zeroizing::new(code.to_owned()))
        .ok_or_else(|| CodedError::new(ErrorCode::ServerNotReachable))
}

/// Whether `code` has the shape of a launch code. The text goes into a page that the browser
/// opens, so anything else (a rogue listener on the port) is refused before it is written.
fn is_launch_code(code: &str) -> bool {
    code.len() == LAUNCH_CODE_CHARS && code.chars().all(|c| c.is_ascii_hexdigit())
}

/// Waits until the server says that no code is waiting any more, but not longer than `lifetime`.
/// A server that stops answering ends the wait too: the file has done its job or cannot.
fn wait_until_taken(port: u16, token: &str, lifetime: Duration) {
    let end = Instant::now() + lifetime;
    while Instant::now() < end {
        sleep(Duration::from_millis(CODE_POLL_MS));
        let pending = call(&Call {
            port,
            method: METHOD_GET,
            path: LAUNCH_CODE_PATH,
            token,
        })
        .ok()
        .and_then(|answer| answer.get(FIELD_PENDING).and_then(Value::as_bool));
        if pending != Some(true) {
            return;
        }
    }
}

/// The private folder of the launch files: below the runtime folder of the user where there is
/// one, else below the data folder (which already is the folder of `CodeSkimr`).
fn launch_dir(locations: &Locations, env: Lookup<'_>) -> PathBuf {
    env(ENV_XDG_RUNTIME_DIR)
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .map_or_else(
            || locations.data_dir.join(LAUNCH_DIR),
            |dir| dir.join(APP_DIR).join(LAUNCH_DIR),
        )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::thread;

    use codeskimr_secrets::KeystoreCredentials;
    use tokio::sync::oneshot;

    use super::*;
    use crate::memory_keystore::Memory;
    use crate::serve::{serve_for_test, test_ui};

    /// The two answers to the exchange and the file the code was in.
    type Seen = (String, String, PathBuf);

    const EXCHANGE: &str = "/api/v1/session/exchange";

    fn post(address: &str, path: &str, body: &str) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn only_a_code_of_the_right_shape_goes_into_the_page() {
        assert!(is_launch_code(&"a1".repeat(16)));
        for text in [
            "",
            "abc",
            &"g".repeat(32),
            "\"><script>x</script>",
            &"a".repeat(33),
        ] {
            assert!(!is_launch_code(text), "{text}");
        }
    }

    /// The code in the redirect page, the way the browser would read it from the address.
    fn code_in(page: &str) -> String {
        let start = page.find("#code=").unwrap() + "#code=".len();
        page[start..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect()
    }

    /// Plays the browser: waits for the launch file, reads the code in it and exchanges it twice.
    /// Returns the two answers and the path of the file.
    fn browser_for(dir: PathBuf, address: String) -> thread::JoinHandle<Seen> {
        thread::spawn(move || {
            let file = loop {
                let found = fs::read_dir(&dir)
                    .unwrap()
                    .flatten()
                    .map(|entry| entry.path())
                    .find(|path| path.extension().is_some_and(|ext| ext == "html"));
                if let Some(path) = found {
                    break path;
                }
                thread::sleep(Duration::from_millis(10));
            };
            let body = format!(
                r#"{{"code":"{}"}}"#,
                code_in(&fs::read_to_string(&file).unwrap())
            );
            let first = post(&address, EXCHANGE, &body);
            let second = post(&address, EXCHANGE, &body);
            (first, second, file)
        })
    }

    /// What the program does while a server runs: `open` and the browser play their parts.
    fn client_side(
        store: &KeystoreCredentials<Memory>,
        url: &str,
        dir: PathBuf,
    ) -> (Result<(), CodedError>, Option<(PathBuf, u64)>, Seen) {
        let address = url.trim_start_matches(URL_SCHEME).to_owned();
        let port = address.rsplit(':').next().unwrap().parse().unwrap();
        let browser = browser_for(dir.clone(), address);
        let plan = Plan {
            port,
            lifetime: Duration::from_secs(30),
            dir,
            browser: false,
        };
        let mut announced = None;
        let outcome = launch(store, &plan, |ready| {
            announced = Some((ready.path.to_owned(), ready.seconds));
        });
        (outcome, announced, browser.join().unwrap())
    }

    #[test]
    fn the_whole_way_from_the_file_to_the_token_and_back_to_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(KeystoreCredentials::new(Memory::default()));
        let (stop, stopped) = oneshot::channel::<()>();
        let (found, url) = std::sync::mpsc::channel::<String>();
        let launch_dir = dir.path().to_path_buf();
        let for_client = Arc::clone(&store);
        let client = thread::spawn(move || {
            let result = client_side(&for_client, &url.recv().unwrap(), launch_dir);
            stop.send(()).unwrap();
            result
        });
        serve_for_test(&*store, test_ui(), &found, stopped).unwrap();
        let (outcome, announced, (first, second, file)) = client.join().unwrap();
        outcome.unwrap();
        assert!(first.starts_with("HTTP/1.1 200"), "{first}");
        assert!(first.contains("csk_"), "{first}");
        assert!(second.starts_with("HTTP/1.1 401"), "once only: {second}");
        assert!(second.contains("API-403"), "{second}");
        assert_eq!(announced, Some((file.clone(), 30)));
        assert!(!file.exists(), "the file goes once the code was taken");
    }

    #[test]
    fn the_launch_folder_is_below_the_runtime_folder_or_the_data_folder() {
        let locations = Locations {
            data_dir: PathBuf::from("/data/codeskimr"),
            languages_dir: PathBuf::new(),
            user_config: None,
            bases: Vec::new(),
        };
        let runtime = |name: &str| (name == ENV_XDG_RUNTIME_DIR).then(|| "/run/user/1".to_owned());
        let none = |_: &str| None;
        assert_eq!(
            launch_dir(&locations, &runtime),
            PathBuf::from("/run/user/1/codeskimr/launch")
        );
        assert_eq!(
            launch_dir(&locations, &none),
            PathBuf::from("/data/codeskimr/launch")
        );
        let relative = |name: &str| (name == ENV_XDG_RUNTIME_DIR).then(|| "tmp".to_owned());
        assert_eq!(
            launch_dir(&locations, &relative),
            PathBuf::from("/data/codeskimr/launch")
        );
    }

    #[test]
    fn without_a_server_the_command_says_so() {
        let store = KeystoreCredentials::new(Memory::default());
        let plan = Plan {
            port: 1,
            lifetime: Duration::from_secs(1),
            dir: PathBuf::new(),
            browser: false,
        };
        let error = launch(&store, &plan, |_| {}).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::ServerNotReachable
        );
    }
}
