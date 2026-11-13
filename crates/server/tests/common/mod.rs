//! A running server with test commands, and a minimal HTTP/1.1 client that can send any header.
#![cfg(test)]
#![allow(dead_code, missing_docs)]

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Registry, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, PermissionSet, Principal};
use codeskimr_server::{CommandApi, InstallationToken, Server};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const TOKEN: &str = "csk_canary-installation-token-0123456789";

/// Reading commands need `read`, the others need the permission they are given.
const fn descriptor(id: &'static str, permission: Permission) -> CommandDescriptor {
    let key = "text";
    CommandDescriptor {
        id: CommandId::new(id),
        description_key: key,
        title_key: key,
        read_only: matches!(permission, Permission::Read),
        shortcut: Some("Mod+T"),
        permission,
    }
}

/// A command that only reads.
pub struct Echo;

impl Command for Echo {
    type Input = String;
    type Output = String;
    const DESCRIPTOR: CommandDescriptor = descriptor("test.echo", Permission::Read);

    fn execute(&self, request: Request<'_, String>) -> Result<String, CodedError> {
        Ok(request.input.to_uppercase())
    }
}

/// A command that changes something and counts its runs.
pub struct Wipe(pub Arc<AtomicUsize>);

impl Command for Wipe {
    type Input = ();
    type Output = usize;
    const DESCRIPTOR: CommandDescriptor = descriptor("test.wipe", Permission::Scan);

    fn execute(&self, _: Request<'_, ()>) -> Result<usize, CodedError> {
        Ok(self.0.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

/// A command that fails with a registered code.
pub struct Missing;

impl Command for Missing {
    type Input = ();
    type Output = ();
    const DESCRIPTOR: CommandDescriptor = descriptor("test.missing", Permission::Read);

    fn execute(&self, _: Request<'_, ()>) -> Result<(), CodedError> {
        Err(CodedError::new(ErrorCode::ProjectNotFound))
    }
}

/// A command that panics.
pub struct Explode;

impl Command for Explode {
    type Input = ();
    type Output = ();
    const DESCRIPTOR: CommandDescriptor = descriptor("test.explode", Permission::Read);

    fn execute(&self, _: Request<'_, ()>) -> Result<(), CodedError> {
        panic!("canary-panic-text")
    }
}

/// A server on a free loopback port.
pub struct Running {
    pub address: SocketAddr,
    pub wipes: Arc<AtomicUsize>,
    pub events: EventBus,
}

/// Starts a server that runs every request as `principal`.
pub async fn start(principal: Principal) -> Running {
    let wipes = Arc::new(AtomicUsize::new(0));
    let mut registry = Registry::new();
    registry.register(Echo).unwrap();
    registry.register(Wipe(Arc::clone(&wipes))).unwrap();
    registry.register(Missing).unwrap();
    registry.register(Explode).unwrap();
    let mut api = CommandApi::new(Arc::new(registry));
    api.expose::<Echo>().unwrap();
    api.expose::<Wipe>().unwrap();
    api.expose::<Missing>().unwrap();
    api.expose::<Explode>().unwrap();
    let events = EventBus::new();
    let server = Server::new(
        api,
        events.clone(),
        principal,
        InstallationToken::from_token(TOKEN),
    );
    let bound = server.bind(([127, 0, 0, 1], 0).into()).await.unwrap();
    let address = bound.address().unwrap();
    tokio::spawn(bound.serve(std::future::pending()));
    Running {
        address,
        wipes,
        events,
    }
}

pub fn owner() -> Principal {
    Principal::limited(PermissionSet::all())
}

pub fn reader() -> Principal {
    Principal::limited(PermissionSet::none().with(Permission::Read))
}

/// What came back.
#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<&str> {
        let wanted = name.to_ascii_lowercase();
        let found = self
            .headers
            .iter()
            .find(|(key, _)| key.to_ascii_lowercase() == wanted);
        found.map(|(_, value)| value.as_str())
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap()
    }

    pub fn code(&self) -> String {
        self.json()["code"].as_str().unwrap().to_owned()
    }
}

/// One request to build.
pub struct Call<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub host: Option<String>,
    pub headers: Vec<(&'a str, String)>,
    pub body: Vec<u8>,
}

impl<'a> Call<'a> {
    pub fn get(path: &'a str) -> Self {
        Self::new("GET", path)
    }

    pub fn post(path: &'a str) -> Self {
        Self::new("POST", path)
    }

    pub fn new(method: &'a str, path: &'a str) -> Self {
        Self {
            method,
            path,
            host: None,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_owned());
        self
    }

    pub fn header(mut self, name: &'a str, value: &str) -> Self {
        self.headers.push((name, value.to_owned()));
        self
    }

    pub fn token(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {token}"))
    }

    pub fn json(mut self, body: &str) -> Self {
        self.headers
            .push(("Content-Type", "application/json".to_owned()));
        self.body = body.as_bytes().to_vec();
        self
    }

    pub fn raw(mut self, content_type: &str, body: &[u8]) -> Self {
        self.headers.push(("Content-Type", content_type.to_owned()));
        self.body = body.to_vec();
        self
    }

    /// Sends the request to `server` and reads the whole answer.
    pub async fn send(self, server: &Running) -> Reply {
        let host = self
            .host
            .clone()
            .unwrap_or_else(|| server.address.to_string());
        let mut lines = vec![
            format!("{} {} HTTP/1.1", self.method, self.path),
            format!("Host: {host}"),
            "Connection: close".to_owned(),
        ];
        lines.extend(
            self.headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}")),
        );
        lines.push(format!("Content-Length: {}", self.body.len()));
        let text = format!("{}\r\n\r\n", lines.join("\r\n"));
        let mut stream = TcpStream::connect(server.address).await.unwrap();
        stream.write_all(text.as_bytes()).await.unwrap();
        stream.write_all(&self.body).await.unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await.unwrap();
        parse(&raw)
    }
}

fn parse(raw: &[u8]) -> Reply {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Reply {
        status,
        headers,
        body: body.to_owned(),
    }
}
