//! A tiny HTTP server in front of `git http-backend`, optionally with Basic authentication.
//!
//! It records which `Authorization` headers it saw, so tests can tell that a login happened.
#![allow(dead_code, missing_docs)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use super::TestResult;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(input: &[u8]) -> String {
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let bits = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                let index = usize::try_from((bits >> (18 - 6 * i)) & 63).unwrap_or(0);
                out.push(BASE64.get(index).map_or('=', |byte| char::from(*byte)));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// An answer that replaces the repository for every request.
#[derive(Clone, Copy)]
enum Fixed {
    /// Redirect to the same path on this port.
    Redirect(u16),
    /// Refuse with this status code and no authentication challenge.
    Status(u16),
}

/// Serves the repositories below `root`; with `login` every request needs those Basic credentials.
pub struct GitHttpServer {
    pub port: u16,
    pub authorizations: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl GitHttpServer {
    pub fn start(root: &Path, login: Option<(&str, &str)>) -> TestResult<Self> {
        Self::launch(root, login, None)
    }

    /// A server that answers every request with a redirect to the same path on `port`.
    pub fn redirecting_to(port: u16) -> TestResult<Self> {
        Self::launch(Path::new(""), None, Some(Fixed::Redirect(port)))
    }

    /// A server that refuses every request with `status`, like a host that hides private
    /// repositories from an invalid token (403 or 404).
    pub fn refusing_with(status: u16) -> TestResult<Self> {
        Self::launch(Path::new(""), None, Some(Fixed::Status(status)))
    }

    fn launch(root: &Path, login: Option<(&str, &str)>, fixed: Option<Fixed>) -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let expected = login.map(|(user, secret)| {
            format!("Basic {}", base64(format!("{user}:{secret}").as_bytes()))
        });
        let stop = Arc::new(AtomicBool::new(false));
        let authorizations = Arc::new(Mutex::new(Vec::new()));
        let (root, flag, seen) = (root.to_path_buf(), stop.clone(), authorizations.clone());
        let thread = std::thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let (root, expected, seen) = (root.clone(), expected.clone(), seen.clone());
                        std::thread::spawn(move || {
                            let _ = serve(stream, &root, expected.as_deref(), fixed, &seen);
                        });
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            }
        });
        Ok(Self {
            port,
            authorizations,
            stop,
            thread: Some(thread),
        })
    }

    /// How many requests carried the expected credentials or any other `Authorization` header.
    pub fn authorization_count(&self) -> usize {
        self.authorizations.lock().map_or(0, |seen| seen.len())
    }
}

impl Drop for GitHttpServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Request {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn read_request(reader: &mut BufReader<TcpStream>) -> TestResult<Request> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or("no method")?.to_owned();
    let target = parts.next().ok_or("no target")?.to_owned();
    let mut headers = Vec::new();
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        let text = line.trim_end();
        if text.is_empty() {
            break;
        }
        if let Some((key, value)) = text.split_once(':') {
            headers.push((key.trim().to_owned(), value.trim().to_owned()));
        }
    }
    let mut request = Request {
        method,
        target,
        headers,
        body: Vec::new(),
    };
    if request
        .header("expect")
        .is_some_and(|v| v.eq_ignore_ascii_case("100-continue"))
    {
        reader
            .get_mut()
            .write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
    }
    request.body = read_body(reader, &request)?;
    Ok(request)
}

fn read_body(reader: &mut BufReader<TcpStream>, request: &Request) -> TestResult<Vec<u8>> {
    let mut body = Vec::new();
    if request
        .header("transfer-encoding")
        .is_some_and(|v| v.eq_ignore_ascii_case("chunked"))
    {
        loop {
            let mut size = String::new();
            reader.read_line(&mut size)?;
            let size = usize::from_str_radix(size.trim(), 16)?;
            if size == 0 {
                let mut end = String::new();
                reader.read_line(&mut end)?;
                break;
            }
            let mut chunk = vec![0; size];
            reader.read_exact(&mut chunk)?;
            body.extend_from_slice(&chunk);
            let mut end = String::new();
            reader.read_line(&mut end)?;
        }
    } else if let Some(length) = request.header("content-length") {
        body.resize(length.parse()?, 0);
        reader.read_exact(&mut body)?;
    }
    Ok(body)
}

fn serve(
    stream: TcpStream,
    root: &Path,
    expected: Option<&str>,
    fixed: Option<Fixed>,
    seen: &Mutex<Vec<String>>,
) -> TestResult {
    let mut reader = BufReader::new(stream.try_clone()?);
    let request = read_request(&mut reader)?;
    let mut stream = stream;
    match fixed {
        Some(Fixed::Redirect(port)) => {
            let location = format!("http://127.0.0.1:{port}{}", request.target);
            stream.write_all(
                format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes(),
            )?;
            return Ok(());
        }
        Some(Fixed::Status(status)) => {
            stream.write_all(
                format!(
                    "HTTP/1.1 {status} Refused\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .as_bytes(),
            )?;
            return Ok(());
        }
        None => {}
    }
    let given = request.header("authorization");
    if let (Some(given), Ok(mut seen)) = (given, seen.lock()) {
        seen.push(given.to_owned());
    }
    if expected.is_some() && given != expected {
        stream.write_all(
            b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"git\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }
    let response = backend(root, &request)?;
    stream.write_all(&response)?;
    Ok(())
}

/// Runs `git http-backend` for the request and turns its CGI answer into an HTTP response.
fn backend(root: &Path, request: &Request) -> TestResult<Vec<u8>> {
    let exec = Command::new("git").arg("--exec-path").output()?;
    let exec = PathBuf::from(String::from_utf8(exec.stdout)?.trim());
    let (path, query) = request
        .target
        .split_once('?')
        .unwrap_or((request.target.as_str(), ""));
    let mut command = Command::new(exec.join("git-http-backend"));
    command
        .env("GIT_PROJECT_ROOT", root)
        .env("GIT_HTTP_EXPORT_ALL", "1")
        .env("REQUEST_METHOD", &request.method)
        .env("PATH_INFO", path)
        .env("QUERY_STRING", query)
        .env("CONTENT_LENGTH", request.body.len().to_string())
        .env("REMOTE_USER", "test")
        .env("REMOTE_ADDR", "127.0.0.1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(kind) = request.header("content-type") {
        command.env("CONTENT_TYPE", kind);
    }
    if let Some(encoding) = request.header("content-encoding") {
        command.env("HTTP_CONTENT_ENCODING", encoding);
    }
    if let Some(protocol) = request.header("git-protocol") {
        command.env("GIT_PROTOCOL", protocol);
    }
    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let body = request.body.clone();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&body);
    });
    let output = child.wait_with_output()?;
    let _ = writer.join();
    cgi_to_http(&output.stdout)
}

fn cgi_to_http(output: &[u8]) -> TestResult<Vec<u8>> {
    let split = output
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("no CGI header")?;
    let (head, rest) = output.split_at(split);
    let head = String::from_utf8_lossy(head).into_owned();
    let body = rest.get(4..).unwrap_or_default();
    let mut status = "200 OK".to_owned();
    let mut response = Vec::new();
    let mut headers = String::new();
    for line in head.lines() {
        if let Some(value) = line.strip_prefix("Status:") {
            value.trim().clone_into(&mut status);
        } else {
            headers.push_str(line);
            headers.push_str("\r\n");
        }
    }
    response.extend_from_slice(
        format!(
            "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .as_bytes(),
    );
    response.extend_from_slice(body);
    Ok(response)
}
