//! A server for the end-to-end tests of the UI (`web/e2e`), started by Playwright:
//! `cargo test -p codeskimr e2e_server -- --ignored --nocapture`.
//!
//! It is the real server with the real launch file and the real exchange, over an in-memory
//! keystore, because a build machine has no keystore of the operating system. Nothing of it is in
//! the program: it exists only in the test build. The protocol is lines on standard output
//! (`E2E_URL=…`, `E2E_LAUNCH_FILE=…`) and on standard input (`launch` creates a new launch file,
//! the end of the input stops the server).

use std::io::{BufRead, Write, stdin, stdout};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use codeskimr_application::events::EventBus;
use codeskimr_cli::GlobalOptions;
use codeskimr_secrets::KeystoreCredentials;
use tokio::sync::oneshot;

use crate::locations::Locations;
use crate::memory_keystore::Memory;
use crate::open::{Plan, launch};
use crate::serve::{Parts, serve_for_test, test_ui};
use crate::ui_commands::command_api;

const UI_DIR_VARIABLE: &str = "CODESKIMR_E2E_UI_DIR";
const LAUNCH_COMMAND: &str = "launch";
const LANGUAGES_PATH: &str = "../../languages";
const CODE_LIFETIME: Duration = Duration::from_secs(60);

/// Where the scans of the tests keep their data: a folder of its own, the packs of the repository.
fn locations(data: &std::path::Path) -> Locations {
    Locations {
        data_dir: data.to_path_buf(),
        languages_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(LANGUAGES_PATH),
        user_config: None,
        bases: Vec::new(),
    }
}

fn say(line: &str) {
    let mut out = stdout().lock();
    writeln!(out, "{line}").unwrap();
    out.flush().unwrap();
}

/// Creates launch files on request and stops the server when the input ends.
fn serve_requests(store: &Arc<KeystoreCredentials<Memory>>, port: u16, stop: oneshot::Sender<()>) {
    let folder = tempfile::tempdir().unwrap();
    for line in stdin().lock().lines().map_while(Result::ok) {
        if line.trim() != LAUNCH_COMMAND {
            continue;
        }
        let store = Arc::clone(store);
        let plan = Plan {
            port,
            lifetime: CODE_LIFETIME,
            dir: folder.path().to_path_buf(),
            browser: false,
        };
        thread::spawn(move || {
            let result = launch(&*store, &plan, |ready| {
                say(&format!("E2E_LAUNCH_FILE={}", ready.path.display()));
            });
            result.unwrap();
        });
    }
    stop.send(()).ok();
}

#[test]
#[ignore = "started by the end-to-end tests of web/"]
fn e2e_server() {
    let ui_dir = PathBuf::from(std::env::var(UI_DIR_VARIABLE).unwrap());
    let events = EventBus::new();
    let mut ui = test_ui();
    ui.dir = Some(ui_dir);
    let data = tempfile::tempdir().unwrap();
    let parts = Parts {
        ui,
        api: command_api(
            &GlobalOptions::default(),
            &locations(data.path()),
            events.clone(),
            crate::memory_keystore::secrets(),
        )
        .unwrap(),
        mcp: None,
        events,
    };
    let store = Arc::new(KeystoreCredentials::new(Memory::default()));
    let (stop, stopped) = oneshot::channel::<()>();
    let (found, url) = channel::<String>();
    let for_requests = Arc::clone(&store);
    let requests = thread::spawn(move || {
        let url = url.recv().unwrap();
        say(&format!("E2E_URL={url}"));
        let port = url.rsplit(':').next().unwrap().parse().unwrap();
        serve_requests(&for_requests, port, stop);
    });
    serve_for_test(&*store, parts, &found, stopped).unwrap();
    requests.join().unwrap();
}
