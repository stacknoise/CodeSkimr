//! A login with a stored credential through the running app (blueprint S-02, issue #184): the
//! credential is stored over REST, a Git project logs in with it at a remote, and the secret
//! reaches that remote and nothing else.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use gix_transport as _;
use serde_json::json;

use crate::security_tests::{
    Folders, Run, assert_answers_clean, command, contains, files, folders, run,
};

const LOGIN_CANARY: &str = "ghp_LOGIN7c1e4a92d0b645f8a3e1d9c2";
const CHALLENGE: &str = "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"canary\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const REFUSAL: &str = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
const AUTHORIZATION: &str = "authorization:";
/// `anton:` and the canary in Base64 (lower-cased like the header lines the remote keeps): the
/// exact login of the stored credential.
const EXPECTED_LOGIN: &str = "basic yw50b246z2hwx0xpr0lon2mxztrhotjkmgi2ndvmogezztfkowmy";

/// A remote that asks for a login and refuses every one; it keeps the `Authorization` headers it
/// was sent. Returns its address and that list.
fn remote() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let keep = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut head = [0_u8; 8192];
            let count = stream.read(&mut head).unwrap_or(0);
            let text = String::from_utf8_lossy(&head[..count]).to_ascii_lowercase();
            let login = text.lines().find(|line| line.starts_with(AUTHORIZATION));
            let answer = if let Some(line) = login {
                keep.lock().unwrap().push(line.to_owned());
                REFUSAL
            } else {
                CHALLENGE
            };
            let _ = stream.write_all(answer.as_bytes());
        }
    });
    (format!("http://{address}/team/repo.git"), seen)
}

#[test]
fn a_stored_credential_reaches_the_remote_and_nothing_else() {
    let folders = folders();
    let (address, seen) = remote();
    let outcome = run(&folders, move |url, token| {
        let stored = json!({ "id": "work", "user": "anton", "secret": LOGIN_CANARY });
        let add = json!({ "source": address, "credential": "work" });
        vec![
            command(url, Some(token), "credentials.set", &stored),
            command(url, Some(token), "projects.add", &add),
            command(url, Some(token), "credentials.list", &json!(null)),
            command(url, Some(token), "projects.list", &json!(null)),
        ]
    });
    // The controls: the credential was stored, used for a login and the remote refused it.
    assert!(outcome.answers[0].contains("200 OK"));
    assert!(
        !outcome.answers[1].contains("200 OK"),
        "the refused login must fail the add"
    );
    assert!(outcome.answers[2].contains("\"stored\":true"));
    let headers = seen.lock().unwrap().clone();
    assert!(!headers.is_empty(), "the remote never saw a login");
    // The secret itself reached the remote, as the login of the stored credential and nothing else.
    assert!(headers.iter().all(|line| line.contains(EXPECTED_LOGIN)));
    assert_nothing_leaked(&outcome, &folders, &headers);
}

/// Nothing the app answered, wrote or kept holds the secret or the token, and the remote never
/// got the installation token.
fn assert_nothing_leaked(outcome: &Run, folders: &Folders, headers: &[String]) {
    let secrets = [LOGIN_CANARY, outcome.token.as_str()];
    assert_answers_clean(&outcome.answers, &secrets);
    for (path, bytes) in files(folders.data.path()) {
        for (kind, secret) in secrets.iter().enumerate() {
            assert!(
                !contains(&bytes, secret),
                "secret {kind} in {}",
                path.display()
            );
        }
    }
    // The remote never got the installation token either.
    assert!(
        headers
            .iter()
            .all(|line| !line.contains(&outcome.token.to_ascii_lowercase()))
    );
}
