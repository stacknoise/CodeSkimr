//! The bare `codeskimr`: the server and the browser start together (ADR 0030).
//!
//! The server runs on its own thread. As soon as it listens, the browser is started. The server
//! keeps running when the browser cannot be started, and the call does not wait for a browser when
//! the server cannot start.

use std::sync::mpsc::{Sender, channel};
use std::thread::scope;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;

/// Runs `serve` on a thread of its own and `listening` once it has told its address through the
/// sender. A failure of `listening` goes to `failed` and ends nothing. Returns when the server
/// has ended, with its failure if it has one.
///
/// # Errors
/// What `serve` returns (for example `API-301` for a taken port), or `APP-501` if its thread
/// panicked.
pub(crate) fn together<E>(
    serve: impl FnOnce(Sender<String>) -> Result<(), CodedError> + Send,
    listening: impl FnOnce(&str) -> Result<(), E>,
    failed: impl FnOnce(E),
) -> Result<(), CodedError> {
    let (sender, address) = channel::<String>();
    scope(|threads| {
        let server = threads.spawn(move || serve(sender));
        // A server that ends without an address drops the sender, so this never waits for good.
        if let Some(failure) = address.recv().ok().and_then(|url| listening(&url).err()) {
            failed(failure);
        }
        server
            .join()
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    use codeskimr_codes::Coded;

    use super::*;

    const WAIT: Duration = Duration::from_secs(10);
    const URL: &str = "http://127.0.0.1:7421";

    /// Runs `call` on another thread, so that a hang fails the test instead of blocking it.
    fn within_time<T: Send + 'static>(call: impl FnOnce() -> T + Send + 'static) -> T {
        let (done, result) = channel();
        std::thread::spawn(move || {
            let _ = done.send(call());
        });
        match result.recv_timeout(WAIT) {
            Ok(value) => value,
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                panic!("the call did not return")
            }
        }
    }

    #[test]
    fn a_server_that_cannot_start_returns_its_code_and_no_browser_is_started() {
        let outcome = within_time(|| {
            let mut browser_started = false;
            let result = together(
                |_address| Err(CodedError::new(ErrorCode::ServerBindFailed)),
                |_url| {
                    browser_started = true;
                    Ok::<(), ()>(())
                },
                |()| (),
            );
            (result, browser_started)
        });
        let (result, browser_started) = outcome;
        assert_eq!(result.unwrap_err().code(), ErrorCode::ServerBindFailed);
        assert!(!browser_started);
    }

    #[test]
    fn a_failing_browser_leaves_the_server_running() {
        let outcome = within_time(|| {
            let (stop, stopped) = channel::<()>();
            let mut failure = None;
            let result = together(
                move |address| {
                    address.send(URL.to_owned()).unwrap();
                    // The server runs until the failure has been reported.
                    stopped
                        .recv()
                        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))
                },
                |_url| Err(ErrorCode::BrowserStartFailed),
                |code| {
                    failure = Some(code);
                    stop.send(()).unwrap();
                },
            );
            (result, failure)
        });
        let (result, failure) = outcome;
        assert!(result.is_ok());
        assert_eq!(failure, Some(ErrorCode::BrowserStartFailed));
    }

    #[test]
    fn the_browser_gets_the_address_of_the_server() {
        let told = within_time(|| {
            let mut told = String::new();
            let result = together(
                |address| {
                    address.send(URL.to_owned()).unwrap();
                    Ok(())
                },
                |url| {
                    url.clone_into(&mut told);
                    Ok::<(), ()>(())
                },
                |()| (),
            );
            (result.is_ok(), told)
        });
        assert_eq!(told, (true, URL.to_owned()));
    }
}
