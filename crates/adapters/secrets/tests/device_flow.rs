//! The OAuth device flow against a scripted local server (blueprint 12.2, point 2).
#![allow(missing_docs)]

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_secrets::{DeviceFlow, DeviceFlowError, Provider};
use secrecy::ExposeSecret;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const DEVICE_CODE: &str = "CANARY-device-code-7c1f";
const TOKEN: &str = "CANARY-token-93ab";

/// Answers the requests one after the other with the scripted bodies and records what it got.
struct Script {
    port: u16,
    requests: Arc<Mutex<Vec<String>>>,
    thread: Option<JoinHandle<()>>,
}

impl Script {
    fn start(answers: Vec<String>) -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = requests.clone();
        let thread = std::thread::spawn(move || {
            for answer in answers {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                let body = respond(stream, &answer).unwrap_or_default();
                if let Ok(mut all) = seen.lock() {
                    all.push(body);
                }
            }
        });
        Ok(Self {
            port,
            requests,
            thread: Some(thread),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    fn bodies(&self) -> Vec<String> {
        if let Some(thread) = &self.thread {
            while !thread.is_finished() {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        self.requests
            .lock()
            .map(|all| all.clone())
            .unwrap_or_default()
    }
}

/// Reads one request, writes `answer` (a complete HTTP response) and returns the request body.
fn respond(stream: TcpStream, answer: &str) -> TestResult<String> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse()?;
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    let mut stream = stream;
    stream.write_all(answer.as_bytes())?;
    Ok(String::from_utf8(body)?)
}

fn json(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn device_reply(interval: u32) -> String {
    json(&format!(
        r#"{{"device_code":"{DEVICE_CODE}","user_code":"WDJB-MJHT","verification_uri":"https://example.com/device","expires_in":900,"interval":{interval}}}"#
    ))
}

fn flow(script: &Script) -> TestResult<DeviceFlow> {
    Ok(DeviceFlow::new(
        script.url("/device"),
        script.url("/token"),
        "client 1",
        "repo",
        "token-user",
    )?)
}

/// The error of a flow that is expected to fail.
fn failure(script: &Script) -> TestResult<DeviceFlowError> {
    flow(script)?
        .sign_in(|_| {}, |_| {})
        .err()
        .ok_or_else(|| "the flow should fail".into())
}

#[test]
fn the_user_approves_and_a_token_comes_back() -> TestResult {
    let script = Script::start(vec![
        device_reply(5),
        json(r#"{"error":"authorization_pending"}"#),
        json(r#"{"error":"slow_down"}"#),
        json(&format!(
            r#"{{"access_token":"{TOKEN}","token_type":"bearer"}}"#
        )),
    ])?;
    let waits = RefCell::new(Vec::new());
    let shown = RefCell::new(None);

    let credential = flow(&script)?.sign_in(
        |prompt| *shown.borrow_mut() = Some(prompt.clone()),
        |pause| waits.borrow_mut().push(pause),
    )?;

    assert_eq!(credential.user().expose_secret(), "token-user");
    assert_eq!(credential.secret().expose_secret(), TOKEN);
    let prompt = shown.borrow().clone().ok_or("no prompt")?;
    assert_eq!(prompt.user_code, "WDJB-MJHT");
    assert_eq!(prompt.verification_uri, "https://example.com/device");
    // The pause before the third poll is five seconds longer: the second answer was `slow_down`.
    let secs: Vec<u64> = waits.borrow().iter().map(Duration::as_secs).collect();
    assert_eq!(secs, [5, 5, 10]);
    Ok(())
}

#[test]
fn the_requests_carry_the_form_fields_escaped() -> TestResult {
    let script = Script::start(vec![
        device_reply(1),
        json(&format!(r#"{{"access_token":"{TOKEN}"}}"#)),
    ])?;
    flow(&script)?.sign_in(|_| {}, |_| {})?;

    let bodies = script.bodies();
    assert_eq!(bodies[0], "client_id=client%201&scope=repo");
    assert_eq!(
        bodies[1],
        format!(
            "client_id=client%201&device_code={DEVICE_CODE}&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code"
        )
    );
    Ok(())
}

#[test]
fn a_denied_sign_in_is_net_601_and_names_no_secret() -> TestResult {
    let script = Script::start(vec![device_reply(1), json(r#"{"error":"access_denied"}"#)])?;

    let error = failure(&script)?;

    assert_eq!(error.code(), ErrorCode::OauthFlowFailed);
    let texts = [error.to_string(), format!("{error:?}")];
    assert!(texts.iter().all(|text| !text.contains(DEVICE_CODE)));
    Ok(())
}

#[test]
fn a_code_that_runs_out_ends_the_flow() -> TestResult {
    let pending = || json(r#"{"error":"authorization_pending"}"#);
    let script = Script::start(vec![
        json(
            r#"{"device_code":"d","user_code":"u","verification_uri":"v","expires_in":2,"interval":1}"#,
        ),
        pending(),
        pending(),
        pending(),
    ])?;
    let error = failure(&script)?;
    assert_eq!(error.code(), ErrorCode::OauthFlowFailed);
    Ok(())
}

#[test]
fn nonsense_from_the_server_is_net_601() -> TestResult {
    let script = Script::start(vec![json("not json")])?;
    let error = failure(&script)?;
    assert_eq!(error.code(), ErrorCode::OauthFlowFailed);
    Ok(())
}

#[test]
fn a_redirect_is_not_followed() -> TestResult {
    let elsewhere = TcpListener::bind("127.0.0.1:0")?;
    elsewhere.set_nonblocking(true)?;
    let target = elsewhere.local_addr()?.port();
    let redirect = format!(
        "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://127.0.0.1:{target}/device\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let script = Script::start(vec![redirect])?;

    let error = failure(&script)?;

    assert_eq!(error.code(), ErrorCode::OauthFlowFailed);
    assert!(elsewhere.accept().is_err());
    Ok(())
}

#[test]
fn only_https_endpoints_are_accepted() {
    let plain = |url: &str| {
        DeviceFlow::new(
            url.to_owned(),
            "https://example.com/token".to_owned(),
            "id",
            "repo",
            "user",
        )
    };
    assert!(plain("http://example.com/device").is_err());
    for host in [
        "http://127.0.0.1.example.com/device",
        "http://127.0.0.1@example.com/device",
        "http://127.0.0.1:80@example.com/device",
        "http://localhost/device",
        "https://user@example.com/device",
    ] {
        assert!(plain(host).is_err(), "{host}");
    }
    assert!(plain("http://127.0.0.1:8080/device").is_ok());
    assert!(plain("https://example.com/device").is_ok());
    assert!(
        Provider::GitHub
            .flow("id", Some("http://ghe.example.com"))
            .is_err()
    );
    assert!(
        Provider::GitLab
            .flow("id", Some("https://git.example.com/"))
            .is_ok()
    );
    assert!(Provider::GitHub.flow("", None).is_err());
}

#[test]
fn absurd_timings_and_control_characters_from_the_server_are_handled() -> TestResult {
    let huge = json(
        r#"{"device_code":"d","user_code":"u","verification_uri":"v","expires_in":18446744073709551615,"interval":18446744073709551615}"#,
    );
    let script = Script::start(vec![
        huge,
        json(r#"{"error":"slow_down"}"#),
        json(r#"{"error":"access_denied"}"#),
    ])?;
    let waits = RefCell::new(Vec::new());
    let result = flow(&script)?.sign_in(|_| {}, |pause| waits.borrow_mut().push(pause));
    assert!(result.is_err());
    assert!(waits.borrow().iter().all(|pause| pause.as_secs() <= 60));

    let escape = json(
        r#"{"device_code":"d","user_code":"A\u001b[2JB","verification_uri":"v","expires_in":900}"#,
    );
    let script = Script::start(vec![escape])?;
    failure(&script)?;

    let empty = json(r#"{"access_token":""}"#);
    let script = Script::start(vec![device_reply(1), empty])?;
    failure(&script)?;
    Ok(())
}

const REFRESH: &str = "CANARY-refresh-5d2e";
const RENEWED: &str = "CANARY-renewed-71c0";
const NEXT_REFRESH: &str = "CANARY-refresh-next-0b9a";

#[test]
fn the_refresh_token_of_a_sign_in_is_kept_and_trades_for_a_new_token() -> TestResult {
    let script = Script::start(vec![
        device_reply(1),
        json(&format!(
            r#"{{"access_token":"{TOKEN}","refresh_token":"{REFRESH}"}}"#
        )),
        json(&format!(
            r#"{{"access_token":"{RENEWED}","refresh_token":"{NEXT_REFRESH}"}}"#
        )),
    ])?;
    let flow = flow(&script)?;

    let signed = flow.sign_in_with_refresh(|_| {}, |_| {})?;
    let refresh = signed.refresh.ok_or("no refresh token")?;
    assert_eq!(refresh.expose_secret(), REFRESH);

    let renewed = flow.refresh(&refresh)?;
    assert_eq!(renewed.credential.user().expose_secret(), "token-user");
    assert_eq!(renewed.credential.secret().expose_secret(), RENEWED);
    let next = renewed.refresh.ok_or("no rotated refresh token")?;
    assert_eq!(next.expose_secret(), NEXT_REFRESH);
    assert_eq!(
        script.bodies()[2],
        format!("client_id=client%201&refresh_token={REFRESH}&grant_type=refresh_token")
    );
    Ok(())
}

#[test]
fn a_refused_refresh_is_net_601_and_names_no_secret() -> TestResult {
    let script = Script::start(vec![json(r#"{"error":"invalid_grant"}"#), json("nonsense")])?;
    let flow = flow(&script)?;
    let token = secrecy::SecretString::from(REFRESH);
    for _ in 0..2 {
        let error = flow.refresh(&token).err().ok_or("should fail")?;
        assert_eq!(error.code(), ErrorCode::OauthFlowFailed);
        let texts = [error.to_string(), format!("{error:?}")];
        assert!(texts.iter().all(|text| !text.contains(REFRESH)));
    }
    Ok(())
}

#[test]
fn a_platform_is_found_again_by_its_name() {
    for provider in [Provider::GitHub, Provider::GitLab] {
        assert_eq!(Provider::from_name(provider.name()), Some(provider));
    }
    assert_eq!(Provider::from_name("bitbucket"), None);
}
