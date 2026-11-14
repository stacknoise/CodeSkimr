//! The decoder: bytes of a SCIP index (`scip.proto` of Sourcegraph) to a [`ScipIndex`].
//!
//! Only the fields the import needs are read; every other field is skipped. The input is
//! untrusted: its size is checked against the limit before it is read, strings are length-checked before they are
//! copied, skipped groups have a depth limit, and nothing here can panic.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use codeskimr_domain::{Position, Span};

use crate::consts::{
    DOCUMENT_OCCURRENCES, DOCUMENT_POSITION_ENCODING, DOCUMENT_RELATIVE_PATH, ENCODING_UTF8,
    ENCODING_UTF16, ENCODING_UTF32, INDEX_DOCUMENTS, INDEX_METADATA, MAX_DOCUMENTS,
    MAX_OCCURRENCES, MAX_PATH_BYTES, MAX_SYMBOL_BYTES, MAX_SYMBOLS, METADATA_PROJECT_ROOT,
    OCCURRENCE_RANGE, OCCURRENCE_ROLES, OCCURRENCE_SYMBOL, RANGE_MULTI_LINE,
};
use crate::error::{Corruption, ScipError};
use crate::model::{Occurrence, PositionEncoding, ScipDocument, ScipIndex, SymbolRef};
use crate::path::normalize_path;
use crate::wire::{Fields, Value, packed_varints};

/// Reads and decodes the index file at `path`.
///
/// The size is checked before the content is read, and the read itself stops one byte past the
/// limit, so a file that grows meanwhile cannot exceed the limit either.
///
/// # Errors
/// [`ScipError::TooLarge`] above `max_index_bytes`, [`ScipError::Unreadable`] if the file cannot
/// be read, and everything [`decode_index`] reports.
pub fn read_index(path: &Path, max_index_bytes: u64) -> Result<ScipIndex, ScipError> {
    let file = File::open(path).map_err(ScipError::Unreadable)?;
    let size = file.metadata().map_err(ScipError::Unreadable)?.len();
    refuse_above(size, max_index_bytes)?;
    let mut bytes = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    file.take(max_index_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(ScipError::Unreadable)?;
    decode_index(&bytes, max_index_bytes)
}

fn refuse_above(size: u64, limit: u64) -> Result<(), ScipError> {
    if size > limit {
        Err(ScipError::TooLarge { size, limit })
    } else {
        Ok(())
    }
}

/// Decodes the bytes of an index.
///
/// # Errors
/// [`ScipError::TooLarge`] if `bytes` is longer than `max_index_bytes`, and
/// [`ScipError::Corrupt`] for truncated or malformed input, text that is not UTF-8 or too long,
/// ranges that are not three or four numbers, and paths that are empty, absolute or hold `..`.
pub fn decode_index(bytes: &[u8], max_index_bytes: u64) -> Result<ScipIndex, ScipError> {
    decode_bounded(bytes, max_index_bytes, &Limits::default())
}

/// Bounds on the number of things an index may hold, so that a small file cannot grow into a
/// large structure in memory.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) documents: usize,
    pub(crate) occurrences: usize,
    pub(crate) symbols: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            documents: MAX_DOCUMENTS,
            occurrences: MAX_OCCURRENCES,
            symbols: MAX_SYMBOLS,
        }
    }
}

pub(crate) fn decode_bounded(
    bytes: &[u8],
    max_index_bytes: u64,
    limits: &Limits,
) -> Result<ScipIndex, ScipError> {
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    refuse_above(size, max_index_bytes)?;
    let mut symbols = Interner::new(limits.symbols);
    let mut index = ScipIndex::default();
    let mut occurrences = 0_usize;
    for field in Fields::new(bytes) {
        match field? {
            (INDEX_METADATA, Value::Bytes(message)) => {
                index.project_root = decode_project_root(message)?;
            }
            (INDEX_DOCUMENTS, Value::Bytes(message)) => {
                let document = decode_document(message, &mut symbols)?;
                occurrences = occurrences.saturating_add(document.occurrences.len());
                admit(index.documents.len(), occurrences, limits)?;
                index.documents.push(document);
            }
            (INDEX_METADATA | INDEX_DOCUMENTS, Value::Varint(_)) => {
                return Err(Corruption::WireType.into());
            }
            _ => {}
        }
    }
    index.symbols = symbols.texts;
    Ok(index)
}

/// Refuses a document that would take the index over the limits.
fn admit(documents: usize, occurrences: usize, limits: &Limits) -> Result<(), Corruption> {
    if documents >= limits.documents {
        Err(Corruption::TooManyDocuments)
    } else if occurrences > limits.occurrences {
        Err(Corruption::TooManyOccurrences)
    } else {
        Ok(())
    }
}

/// Hands out one handle per distinct symbol text.
struct Interner {
    texts: Vec<String>,
    handles: HashMap<String, u32>,
    limit: usize,
}

impl Interner {
    fn new(limit: usize) -> Self {
        Self {
            texts: Vec::new(),
            handles: HashMap::new(),
            limit,
        }
    }

    fn intern(&mut self, text: &str) -> Result<SymbolRef, Corruption> {
        if let Some(handle) = self.handles.get(text) {
            return Ok(SymbolRef(*handle));
        }
        if self.texts.len() >= self.limit {
            return Err(Corruption::TooManySymbols);
        }
        let handle = u32::try_from(self.texts.len()).map_err(|_| Corruption::TooManySymbols)?;
        self.texts.push(text.to_owned());
        self.handles.insert(text.to_owned(), handle);
        Ok(SymbolRef(handle))
    }
}

fn text(bytes: &[u8], limit: usize) -> Result<&str, Corruption> {
    if bytes.len() > limit {
        return Err(Corruption::TextTooLong);
    }
    std::str::from_utf8(bytes).map_err(|_| Corruption::NotUtf8)
}

fn decode_project_root(message: &[u8]) -> Result<String, Corruption> {
    let mut root = String::new();
    for field in Fields::new(message) {
        match field? {
            (METADATA_PROJECT_ROOT, Value::Bytes(bytes)) => {
                text(bytes, MAX_PATH_BYTES)?.clone_into(&mut root);
            }
            (METADATA_PROJECT_ROOT, Value::Varint(_)) => return Err(Corruption::WireType),
            _ => {}
        }
    }
    Ok(root)
}

fn decode_document(message: &[u8], symbols: &mut Interner) -> Result<ScipDocument, Corruption> {
    let mut path = None;
    let mut encoding = PositionEncoding::default();
    let mut occurrences = Vec::new();
    for field in Fields::new(message) {
        match field? {
            (DOCUMENT_RELATIVE_PATH, Value::Bytes(bytes)) => {
                path = Some(normalize_path(text(bytes, MAX_PATH_BYTES)?)?);
            }
            (DOCUMENT_OCCURRENCES, Value::Bytes(bytes)) => {
                occurrences.extend(decode_occurrence(bytes, symbols)?);
            }
            (DOCUMENT_POSITION_ENCODING, Value::Varint(value)) => encoding = encoding_of(value),
            (DOCUMENT_RELATIVE_PATH | DOCUMENT_OCCURRENCES, Value::Varint(_))
            | (DOCUMENT_POSITION_ENCODING, Value::Bytes(_)) => {
                return Err(Corruption::WireType);
            }
            _ => {}
        }
    }
    Ok(ScipDocument {
        path: path.ok_or(Corruption::UnsafePath)?,
        encoding,
        occurrences,
    })
}

const fn encoding_of(value: u64) -> PositionEncoding {
    match value {
        ENCODING_UTF8 => PositionEncoding::Utf8,
        ENCODING_UTF16 => PositionEncoding::Utf16,
        ENCODING_UTF32 => PositionEncoding::Utf32,
        _ => PositionEncoding::Unspecified,
    }
}

/// The up to four numbers of `Occurrence.range`.
#[derive(Default)]
struct RangeNumbers {
    values: [u32; RANGE_MULTI_LINE],
    len: usize,
}

impl RangeNumbers {
    fn push(&mut self, raw: u64) -> Result<(), Corruption> {
        let value = u32::try_from(raw).map_err(|_| Corruption::NumberOutOfRange)?;
        let slot = self.values.get_mut(self.len).ok_or(Corruption::BadRange)?;
        *slot = value;
        self.len += 1;
        Ok(())
    }

    fn span(&self) -> Result<Span, Corruption> {
        let at = |line, column| Position { line, column };
        let (start, end) = match self.values.get(..self.len) {
            Some(&[line, start, end]) => (at(line, start), at(line, end)),
            Some(&[first, start, last, end]) => (at(first, start), at(last, end)),
            _ => return Err(Corruption::BadRange),
        };
        Span::new(start, end).map_err(|_| Corruption::BadRange)
    }
}

/// An occurrence without a symbol says nothing and is dropped; `None` then.
fn decode_occurrence(
    message: &[u8],
    symbols: &mut Interner,
) -> Result<Option<Occurrence>, Corruption> {
    let mut range = RangeNumbers::default();
    let mut symbol = None;
    let mut roles = 0;
    for field in Fields::new(message) {
        match field? {
            (OCCURRENCE_RANGE, Value::Varint(number)) => range.push(number)?,
            (OCCURRENCE_RANGE, Value::Bytes(bytes)) => {
                for number in packed_varints(bytes) {
                    range.push(number?)?;
                }
            }
            (OCCURRENCE_SYMBOL, Value::Bytes(bytes)) => {
                symbol = Some(symbols.intern(text(bytes, MAX_SYMBOL_BYTES)?)?);
            }
            (OCCURRENCE_ROLES, Value::Varint(value)) => {
                roles = u32::try_from(value).map_err(|_| Corruption::NumberOutOfRange)?;
            }
            (OCCURRENCE_SYMBOL, Value::Varint(_)) | (OCCURRENCE_ROLES, Value::Bytes(_)) => {
                return Err(Corruption::WireType);
            }
            _ => {}
        }
    }
    let range = range.span()?;
    Ok(symbol.map(|symbol| Occurrence {
        range,
        symbol,
        roles,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document `a.rs` with one definition each of `count` distinct symbols.
    fn index_of(documents: usize, per_document: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for number in 0..documents {
            let mut document = field(DOCUMENT_RELATIVE_PATH, format!("d{number}.rs").as_bytes());
            for item in 0..per_document {
                let mut occurrence = field(OCCURRENCE_RANGE, &[0, 1, 2]);
                occurrence.extend(field(
                    OCCURRENCE_SYMBOL,
                    format!("local {number}-{item}").as_bytes(),
                ));
                document.extend(field(DOCUMENT_OCCURRENCES, &occurrence));
            }
            bytes.extend(field(INDEX_DOCUMENTS, &document));
        }
        bytes
    }

    fn field(number: u32, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![u8::try_from((number << 3) | 2).expect("small tag")];
        let mut length = payload.len();
        while length >= 0x80 {
            bytes.push(u8::try_from(length & 0x7f).expect("seven bits") | 0x80);
            length >>= 7;
        }
        bytes.push(u8::try_from(length).expect("last seven bits"));
        bytes.extend(payload);
        bytes
    }

    fn failure(bytes: &[u8], limits: &Limits) -> Option<Corruption> {
        match decode_bounded(bytes, u64::MAX, limits) {
            Err(ScipError::Corrupt(detail)) => Some(detail),
            _ => None,
        }
    }

    fn limits(documents: usize, occurrences: usize, symbols: usize) -> Limits {
        Limits {
            documents,
            occurrences,
            symbols,
        }
    }

    #[test]
    fn counts_above_the_limits_are_refused() {
        let roomy = 1000;
        assert_eq!(
            failure(&index_of(3, 1), &limits(2, roomy, roomy)),
            Some(Corruption::TooManyDocuments)
        );
        assert_eq!(
            failure(&index_of(2, 3), &limits(roomy, 4, roomy)),
            Some(Corruption::TooManyOccurrences)
        );
        assert_eq!(
            failure(&index_of(2, 2), &limits(roomy, roomy, 3)),
            Some(Corruption::TooManySymbols)
        );
        assert_eq!(failure(&index_of(2, 2), &limits(2, 4, 4)), None);
    }

    #[test]
    fn many_distinct_symbols_are_stored_once_each() {
        let index = decode_index(&index_of(50, 100), u64::MAX).expect("decodes");
        assert_eq!(index.symbols.len(), 5000);
    }
}
