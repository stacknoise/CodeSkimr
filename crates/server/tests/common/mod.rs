//! A running server with test commands, and a minimal HTTP/1.1 client that can send any header.
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
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const TOKEN: &str = "csk_canary-installation-token-0123456789";

#[derive(Deserialize)]
pub struct EchoInput {
    pub text: String,
}

#[derive(Serialize)]
pub struct EchoOutput {
    pub echo: String,
}

/// A command that only reads.
pub struct Echo;

impl Command for Echo {
    type Input = EchoInput;
    type Output = EchoOutput;
    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new("test.echo"),
        title_key: "title",
        description_key: "description",
        shortcut: Some("Mod+E"),
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, EchoInput>) -> Result<EchoOutput, CodedError> {
        Ok(EchoOutput {
            echo: request.input.text,
        })
    }
}

/// A command that changes something and counts its runs.
pub struct Wipe(pub Arc<AtomicUsize>);

impl Command for Wipe {
    type Input = ();
    type Output = usize;
    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new("test.wipe"),
        title_key: "title",
        description_key: "description",
        shortcut: None,
        read_only: false,
        permission: Permission::Scan,
    };

    fn execute(&self, _: Request<'_, ()>) -> Result<usize, CodedError> {
        Ok(self.0.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

/// A command that fails with a registered code.
pub struct Missing;

impl Command for Missing {
    type Input = ();
    type Output = ();
    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new("test.missing"),
        title_key: "title",
        description_key: "description",
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, _: Request<'_, ()>) -> Result<(), CodedError> {
        Err(CodedError::new(ErrorCode::ProjectNotFound))
    }
}

/// A command that panics.
pub struct Explode;

impl Command for Explode {
    type Input = ();
    type Output = ();
    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new("test.explode"),
        title_key: "title",
        description_key: "description",
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

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
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
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
        let mut text = format!(
            "{} {} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n",
            self.method, self.path
        );
        for (name, value) in &self.headers {
            text.push_str(&format!("{name}: {value}\r\n"));
        }
        text.push_str(&format!("Content-Length: {}\r\n\r\n", self.body.len()));
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
