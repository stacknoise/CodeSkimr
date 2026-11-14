//! Converts the columns of an index to UTF-8 bytes, the unit of the parsing adapter (issue #217).
//!
//! scip-dotnet and scip-typescript count UTF-16 code units, others count characters; `CodeSkimr`
//! counts bytes. They agree on lines without characters beyond ASCII before the column and
//! differ elsewhere, so the stored span of a reference and the lookup of the level 1 kind at the
//! same place would be off. The conversion needs the text of the lines.

use std::collections::HashMap;

use codeskimr_domain::{Position, RelativePath, Span};

use crate::consts::MAX_LOCAL_NAME_BYTES;
use crate::model::{Occurrence, PositionEncoding, ScipDocument, ScipIndex, SymbolRef};
use crate::symbol::is_local;

/// Counts how many units of `encoding` a character takes.
fn units_of(encoding: PositionEncoding, character: char) -> usize {
    match encoding {
        PositionEncoding::Utf32 => 1,
        PositionEncoding::Utf8 => character.len_utf8(),
        PositionEncoding::Utf16 | PositionEncoding::Unspecified => character.len_utf16(),
    }
}

/// The byte offset in `line` of the place `column` units from its start. A column inside a
/// surrogate pair stands for the next character. `None` if the column lies behind the
/// end of the line.
fn byte_column(line: &str, column: u32, encoding: PositionEncoding) -> Option<u32> {
    if line.is_ascii() {
        let length = u32::try_from(line.len()).ok()?;
        return (column <= length).then_some(column);
    }
    let wanted = usize::try_from(column).ok()?;
    let mut units: usize = 0;
    for (offset, character) in line.char_indices() {
        if units >= wanted {
            return u32::try_from(offset).ok();
        }
        units = units.saturating_add(units_of(encoding, character));
    }
    (units >= wanted).then(|| u32::try_from(line.len()).ok())?
}

fn convert_position(
    lines: &[String],
    position: Position,
    encoding: PositionEncoding,
) -> Option<Position> {
    let line = lines.get(usize::try_from(position.line).ok()?)?;
    let column = byte_column(line, position.column, encoding)?;
    Some(Position {
        line: position.line,
        column,
    })
}

fn convert_occurrence(
    occurrence: &Occurrence,
    lines: &[String],
    encoding: PositionEncoding,
) -> Option<Occurrence> {
    let start = convert_position(lines, occurrence.range.start(), encoding)?;
    let end = convert_position(lines, occurrence.range.end(), encoding)?;
    Some(Occurrence {
        range: Span::new(start, end).ok()?,
        ..*occurrence
    })
}

fn convert_document(document: &mut ScipDocument, lines: &[String]) {
    let encoding = document.encoding;
    // An occurrence that cannot be placed in the text (a column behind the end of its line) is
    // dropped: the file changed since the index was made, and a guessed span would point at the
    // wrong place. Only then can the whole document be called UTF-8.
    document.occurrences = std::mem::take(&mut document.occurrences)
        .iter()
        .filter_map(|occurrence| convert_occurrence(occurrence, lines, encoding))
        .collect();
    document.encoding = PositionEncoding::Utf8;
}

/// The text a one-line range covers in the converted (byte) columns of `lines`.
fn text_at(lines: &[String], range: Span) -> Option<&str> {
    let (start, end) = (range.start(), range.end());
    if start.line != end.line || end.column.checked_sub(start.column)? > MAX_LOCAL_NAME_BYTES {
        return None;
    }
    let line = lines.get(usize::try_from(start.line).ok()?)?;
    line.get(usize::try_from(start.column).ok()?..usize::try_from(end.column).ok()?)
}

/// The names of the local symbols a document defines: the text of their definitions. A symbol
/// whose definitions do not read the same gets none. Needs columns in bytes.
fn local_names(
    document: &ScipDocument,
    lines: &[String],
    symbols: &[String],
) -> HashMap<SymbolRef, String> {
    let mut names: HashMap<SymbolRef, Option<&str>> = HashMap::new();
    let definitions = document.occurrences.iter().filter(|o| o.is_definition());
    for occurrence in definitions {
        let is_local_symbol = usize::try_from(occurrence.symbol.0)
            .ok()
            .and_then(|at| symbols.get(at))
            .is_some_and(|text| is_local(text));
        if !is_local_symbol {
            continue;
        }
        let text = text_at(lines, occurrence.range);
        let known = names.entry(occurrence.symbol).or_insert(text);
        if *known != text {
            *known = None;
        }
    }
    names
        .into_iter()
        .filter_map(|(symbol, text)| Some((symbol, text?.to_owned())))
        .collect()
}

/// Whether the document defines a local symbol, whose name the import reads from the text.
fn defines_locals(document: &ScipDocument, symbols: &[String]) -> bool {
    document.occurrences.iter().any(|occurrence| {
        occurrence.is_definition()
            && usize::try_from(occurrence.symbol.0)
                .ok()
                .and_then(|at| symbols.get(at))
                .is_some_and(|text| is_local(text))
    })
}

/// Turns the columns of every document that does not count UTF-8 bytes into UTF-8 bytes, and
/// reads the names of the local symbols a document defines (the import has no other way to
/// tell what a local symbol is called). `lines_of` gives the lines of a file without their line
/// ends, or `None` if the text is not available, in which case the document keeps its encoding
/// and its local symbols stay without a name. It must read files the way the import is allowed
/// to: inside the project and never through a link.
pub fn convert_columns(
    index: &mut ScipIndex,
    lines_of: &mut dyn FnMut(&RelativePath) -> Option<Vec<String>>,
) {
    let ScipIndex {
        documents, symbols, ..
    } = index;
    for document in documents {
        let converted = document.encoding == PositionEncoding::Utf8;
        if converted && !defines_locals(document, symbols) {
            continue;
        }
        let Some(lines) = lines_of(&document.path) else {
            continue;
        };
        if !converted {
            convert_document(document, &lines);
        }
        document.local_names = local_names(document, &lines, symbols);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SymbolRef;

    fn span(line: u32, from: u32, to: u32) -> Span {
        Span::new(
            Position { line, column: from },
            Position { line, column: to },
        )
        .unwrap()
    }

    fn index(encoding: PositionEncoding, range: Span) -> ScipIndex {
        ScipIndex {
            documents: vec![ScipDocument {
                path: RelativePath::new("a.cs").unwrap(),
                encoding,
                local_names: HashMap::new(),
                occurrences: vec![Occurrence {
                    range,
                    symbol: SymbolRef(0),
                    roles: 0,
                }],
            }],
            ..ScipIndex::default()
        }
    }

    fn convert(encoding: PositionEncoding, line: &str, range: Span) -> Span {
        let mut index = index(encoding, range);
        let lines = vec![line.to_owned()];
        convert_columns(&mut index, &mut |_| Some(lines.clone()));
        index.documents[0].occurrences[0].range
    }

    #[test]
    fn utf16_columns_become_bytes_after_umlauts_and_astral_characters() {
        // "ä" is one UTF-16 unit and two bytes; the emoji is two units and four bytes.
        let line = "ä😀 Foo();";
        assert_eq!(
            convert(PositionEncoding::Utf16, line, span(0, 4, 7)),
            span(0, 7, 10)
        );
        assert_eq!(
            convert(PositionEncoding::Utf32, line, span(0, 3, 6)),
            span(0, 7, 10)
        );
    }

    #[test]
    fn ascii_lines_and_utf8_documents_keep_their_columns() {
        assert_eq!(
            convert(PositionEncoding::Utf16, "int Foo;", span(0, 4, 7)),
            span(0, 4, 7)
        );
        assert_eq!(
            convert(PositionEncoding::Utf8, "ä Foo;", span(0, 3, 6)),
            span(0, 3, 6)
        );
    }

    #[test]
    fn a_column_behind_the_line_drops_the_occurrence() {
        let mut index = index(PositionEncoding::Utf16, span(0, 4, 9));
        let lines = vec!["ä".to_owned()];
        convert_columns(&mut index, &mut |_| Some(lines.clone()));
        assert_eq!(index.documents[0].occurrences, Vec::new());
        assert_eq!(index.documents[0].encoding, PositionEncoding::Utf8);
    }

    #[test]
    fn a_missing_text_leaves_the_document_alone() {
        let mut index = index(PositionEncoding::Utf16, span(0, 1, 2));
        convert_columns(&mut index, &mut |_| None);
        assert_eq!(index.documents[0].encoding, PositionEncoding::Utf16);
        assert_eq!(index.documents[0].occurrences.len(), 1);
    }
}
