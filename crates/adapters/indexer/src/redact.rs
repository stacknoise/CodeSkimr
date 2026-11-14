//! Redacting and shortening text that came from an indexer or a tool.

/// Masks secrets in text that is about to be logged or shown.
pub type Redact = fn(&str) -> String;

/// The last `max_bytes` of `text`, after redaction. Redaction comes first: a secret that
/// crosses the cut would otherwise be cut in half and no longer be recognised.
#[must_use]
pub fn excerpt(text: &str, max_bytes: usize, redact: Redact) -> String {
    let masked = redact(text);
    let start = masked.len().saturating_sub(max_bytes);
    masked
        .char_indices()
        .map(|(index, _)| index)
        .find(|index| *index >= start)
        .and_then(|index| masked.get(index..))
        .unwrap_or_default()
        .to_owned()
}
