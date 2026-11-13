//! The output side of the log: every byte that leaves the process passes through [`redact`].

use std::io::{self, Write};

use crate::redact::redact;

/// Collects what is written and hands it to `inner` with all secrets masked.
///
/// Nothing is passed on before `flush` or the drop, so a secret split over several writes is still
/// seen as a whole. The log subscriber creates one writer per log line and drops it afterwards.
pub struct RedactingWriter<W: Write> {
    inner: W,
    pending: Vec<u8>,
}

impl<W: Write> RedactingWriter<W> {
    /// Wraps `inner`.
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            pending: Vec::new(),
        }
    }

    fn pass_on(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&self.pending);
        let masked = redact(&text);
        self.pending.clear();
        self.inner.write_all(masked.as_bytes())
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.pass_on()?;
        self.inner.flush()
    }
}

impl<W: Write> Drop for RedactingWriter<W> {
    fn drop(&mut self) {
        // A log line that cannot be written has nobody to report to.
        let _ = self.pass_on().and_then(|()| self.inner.flush());
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::RedactingWriter;

    #[test]
    fn masks_a_secret_split_over_several_writes() {
        let mut out = Vec::new();
        {
            let mut writer = RedactingWriter::new(&mut out);
            writer.write_all(b"login password=CAN").unwrap();
            writer.write_all(b"ARY123 done\n").unwrap();
        }
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "login password=[redacted] done\n"
        );
    }

    #[test]
    fn flush_passes_pending_text_on() {
        let mut out = Vec::new();
        let mut writer = RedactingWriter::new(&mut out);
        writer.write_all(b"plain").unwrap();
        writer.flush().unwrap();
        drop(writer);
        assert_eq!(out, b"plain");
    }
}
