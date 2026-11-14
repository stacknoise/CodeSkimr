//! A scripted OAuth server on the loopback address, for the tests of the sign-in and the refresh.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// What the server received, one entry per request.
pub type Requests = Arc<Mutex<Vec<String>>>;

/// Starts a server that answers the requests one after the other with the JSON bodies in
/// `answers`; the last one is repeated. Returns its address (`http://127.0.0.1:port`) and the
/// requests it got.
pub fn serve(answers: &[&str]) -> (String, Requests) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let seen: Requests = Arc::default();
    let keep = Arc::clone(&seen);
    let bodies: Vec<String> = answers.iter().map(|body| (*body).to_owned()).collect();
    std::thread::spawn(move || {
        for (index, stream) in listener.incoming().flatten().enumerate() {
            let mut stream = stream;
            let mut buffer = [0_u8; 8192];
            let count = stream.read(&mut buffer).unwrap_or(0);
            keep.lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buffer[..count]).into_owned());
            let body = &bodies[index.min(bodies.len() - 1)];
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    (host, seen)
}
