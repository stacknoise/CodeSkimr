//! Decoding of file bytes into text (UTF-8 with or without BOM, UTF-16 with BOM).

use crate::consts::{UTF8_BOM, UTF16_BE_BOM, UTF16_LE_BOM, UTF16_UNIT_BYTES};
use crate::error::ParsingError;

/// Decodes `bytes` into text.
///
/// # Errors
/// [`ParsingError::EncodingUnsupported`] if the bytes are neither valid UTF-8 nor UTF-16 with a BOM.
pub fn decode(bytes: &[u8]) -> Result<String, ParsingError> {
    if let Some(rest) = bytes.strip_prefix(UTF16_LE_BOM) {
        return decode_utf16(rest, u16::from_le_bytes);
    }
    if let Some(rest) = bytes.strip_prefix(UTF16_BE_BOM) {
        return decode_utf16(rest, u16::from_be_bytes);
    }
    let body = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    String::from_utf8(body.to_vec()).map_err(|_| ParsingError::EncodingUnsupported)
}

fn decode_utf16(
    bytes: &[u8],
    unit: fn([u8; UTF16_UNIT_BYTES]) -> u16,
) -> Result<String, ParsingError> {
    let mut units = Vec::with_capacity(bytes.len() / UTF16_UNIT_BYTES);
    for pair in bytes.chunks(UTF16_UNIT_BYTES) {
        let pair: [u8; UTF16_UNIT_BYTES] = pair
            .try_into()
            .map_err(|_| ParsingError::EncodingUnsupported)?;
        units.push(unit(pair));
    }
    String::from_utf16(&units).map_err(|_| ParsingError::EncodingUnsupported)
}
