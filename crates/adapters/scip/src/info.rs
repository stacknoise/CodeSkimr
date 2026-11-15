//! `SymbolInformation` of a document: what the index states about the symbols it defines (kind,
//! display name, signature, documentation). Level 2 creates symbols from it (ADR 0055), so every
//! text is bounded: an oversized text leaves the symbol without information instead of failing
//! the index, and all texts of an index together stay within a budget.

use crate::consts::{
    DOC_JOIN, DOCUMENT_TEXT, INFO_DISPLAY_NAME, INFO_DOCUMENTATION, INFO_KIND, INFO_SIGNATURE,
    INFO_SYMBOL, MAX_INFO_DOC_BYTES, MAX_INFO_NAME_BYTES, MAX_INFO_SIGNATURE_BYTES,
    MAX_INFO_TOTAL_BYTES, MAX_SYMBOL_BYTES,
};
use crate::error::Corruption;
use crate::model::{SymbolInfo, SymbolRef};
use crate::wire::{Fields, Value};

/// How many bytes of information texts the decoder has kept so far.
#[derive(Debug, Default)]
pub(crate) struct InfoBudget {
    used: usize,
}

impl InfoBudget {
    /// Takes `bytes` from the budget; `false` if it is used up.
    fn take(&mut self, bytes: usize) -> bool {
        let total = self.used.saturating_add(bytes);
        let fits = total <= MAX_INFO_TOTAL_BYTES;
        if fits {
            self.used = total;
        }
        fits
    }
}

/// The text of `bytes` if it is valid UTF-8 and at most `limit` long; else `None`.
fn bounded(bytes: &[u8], limit: usize) -> Option<&str> {
    if bytes.len() > limit {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}

/// The `text` of the `Document` that carries the signature.
fn signature_text(message: &[u8]) -> Result<Option<String>, Corruption> {
    for field in Fields::new(message) {
        if let (DOCUMENT_TEXT, Value::Bytes(bytes)) = field? {
            let text = bounded(bytes, MAX_INFO_SIGNATURE_BYTES).filter(|text| !text.is_empty());
            return Ok(text.map(str::to_owned));
        }
    }
    Ok(None)
}

/// The parts read so far of one `SymbolInformation`.
#[derive(Default)]
struct Parts {
    symbol: Option<String>,
    kind: u32,
    display_name: String,
    signature: Option<String>,
    documentation: Vec<String>,
    doc_bytes: usize,
}

impl Parts {
    fn add_documentation(&mut self, bytes: &[u8]) {
        let Some(part) = bounded(bytes, MAX_INFO_DOC_BYTES).filter(|part| !part.is_empty()) else {
            return;
        };
        self.doc_bytes = self.doc_bytes.saturating_add(part.len());
        if self.doc_bytes <= MAX_INFO_DOC_BYTES {
            self.documentation.push(part.to_owned());
        }
    }

    fn take(&mut self, field: u32, value: Value<'_>) -> Result<(), Corruption> {
        match (field, value) {
            (INFO_SYMBOL, Value::Bytes(bytes)) => {
                self.symbol = bounded(bytes, MAX_SYMBOL_BYTES).map(str::to_owned);
            }
            (INFO_KIND, Value::Varint(number)) => {
                self.kind = u32::try_from(number).unwrap_or_default();
            }
            (INFO_DISPLAY_NAME, Value::Bytes(bytes)) => {
                bounded(bytes, MAX_INFO_NAME_BYTES)
                    .unwrap_or_default()
                    .clone_into(&mut self.display_name);
            }
            (INFO_SIGNATURE, Value::Bytes(bytes)) => self.signature = signature_text(bytes)?,
            (INFO_DOCUMENTATION, Value::Bytes(bytes)) => self.add_documentation(bytes),
            (
                INFO_SYMBOL | INFO_DISPLAY_NAME | INFO_SIGNATURE | INFO_DOCUMENTATION,
                Value::Varint(_),
            )
            | (INFO_KIND, Value::Bytes(_)) => return Err(Corruption::WireType),
            _ => {}
        }
        Ok(())
    }

    fn bytes(&self) -> usize {
        let signature = self.signature.as_ref().map_or(0, String::len);
        self.display_name.len() + signature + self.doc_bytes
    }
}

/// Decodes one `SymbolInformation`. `intern` gives the handle of the symbol text. `None` for one
/// without a symbol, without a usable text, or when the budget of the index is used up.
///
/// # Errors
/// [`Corruption`] for a field with the wrong wire type or a truncated message.
pub(crate) fn decode_info(
    message: &[u8],
    budget: &mut InfoBudget,
    intern: &mut dyn FnMut(&str) -> Result<SymbolRef, Corruption>,
) -> Result<Option<SymbolInfo>, Corruption> {
    let mut parts = Parts::default();
    for field in Fields::new(message) {
        let (number, value) = field?;
        parts.take(number, value)?;
    }
    let Some(text) = parts.symbol.as_deref() else {
        return Ok(None);
    };
    if !budget.take(parts.bytes()) {
        return Ok(None);
    }
    let documentation =
        (!parts.documentation.is_empty()).then(|| parts.documentation.join(DOC_JOIN));
    Ok(Some(SymbolInfo {
        symbol: intern(text)?,
        kind: parts.kind,
        display_name: parts.display_name,
        signature: parts.signature,
        documentation,
    }))
}
