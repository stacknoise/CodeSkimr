//! Logging: one `tracing` subscriber that writes to standard error through the redaction layer
//! (blueprint S-01), and a panic hook that reports through the same path.
//!
//! Standard output stays free for command results and the MCP protocol.

use std::io::{self, Write};
use std::panic::PanicHookInfo;

use codeskimr_codes::ErrorCode;
use tracing::Subscriber;
use tracing_subscriber::filter::LevelFilter;

use crate::consts::{DEFAULT_LOG_LEVEL, ENV_LOG};
use crate::locations::Lookup;
use crate::log_writer::RedactingWriter;

/// Installs the subscriber and the panic hook. The level comes from `CODESKIMR_LOG`.
///
/// Calling it again keeps the subscriber that is already installed.
pub fn init(env: Lookup<'_>) {
    let level = env(ENV_LOG)
        .and_then(|value| value.parse::<LevelFilter>().ok())
        .unwrap_or(DEFAULT_LOG_LEVEL);
    // Another subscriber may be installed already; then it stays, and so does the panic hook.
    if tracing::subscriber::set_global_default(subscriber(level, io::stderr)).is_ok() {
        std::panic::set_hook(Box::new(report_panic));
    }
}

/// A subscriber that writes every log line through a [`RedactingWriter`] around `sink()`.
pub fn subscriber<W>(
    level: LevelFilter,
    sink: impl Fn() -> W + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync
where
    W: Write + 'static,
{
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(false)
        .with_writer(move || RedactingWriter::new(sink()))
        .finish()
}

/// Logs a panic as an error event instead of the unfiltered text of the default hook.
fn report_panic(info: &PanicHookInfo<'_>) {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(ToString::to_string)
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_default();
    let location = info.location().map(ToString::to_string).unwrap_or_default();
    tracing::error!(code = ErrorCode::UnexpectedInternalError.id(), %location, %message);
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::filter::LevelFilter;

    use super::{report_panic, subscriber};

    const CANARY: &str = "CANARY7f3a9c1e5b2d4f60";

    /// Collects log output in memory.
    #[derive(Clone, Default)]
    struct Sink(Arc<Mutex<Vec<u8>>>);

    impl Write for Sink {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Sink {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    fn logged(level: LevelFilter, emit: impl FnOnce()) -> String {
        let sink = Sink::default();
        let copy = sink.clone();
        tracing::subscriber::with_default(subscriber(level, move || copy.clone()), emit);
        sink.text()
    }

    #[test]
    fn canaries_in_messages_and_fields_never_reach_the_output() {
        let text = logged(LevelFilter::TRACE, || {
            tracing::error!("clone https://bob:{CANARY}@host.example/r.git failed");
            tracing::warn!(header = %format!("Authorization: Bearer {CANARY}"), "request");
            tracing::info!(password = CANARY, "login");
            tracing::debug!(error = ?format!("token={CANARY}"), "denied");
            tracing::trace!(note = %format!("ghp_{CANARY}"));
        });
        assert!(!text.contains(CANARY), "leaked: {text}");
        assert_eq!(text.matches("[redacted]").count(), 4, "{text}");
    }

    #[test]
    fn level_filters_events() {
        let text = logged(LevelFilter::WARN, || {
            tracing::info!("quiet");
            tracing::warn!("loud");
        });
        assert!(text.contains("loud") && !text.contains("quiet"));
    }

    #[test]
    fn a_panic_is_reported_without_its_secret() {
        // The hook is process-wide. This is the only test that replaces it, and it restores the
        // previous hook right after, so other tests lose at most the panic text of a failure.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(report_panic));
        let text = logged(LevelFilter::ERROR, || {
            let result = std::panic::catch_unwind(|| {
                panic!("login failed with password={CANARY}");
            });
            assert!(result.is_err());
        });
        std::panic::set_hook(previous);
        assert!(!text.contains(CANARY), "leaked: {text}");
        assert!(
            text.contains("APP-901") && text.contains("logging.rs"),
            "{text}"
        );
    }
}
