//! Conversion of tree-sitter positions into domain spans and text.

use codeskimr_domain::{Position, Span};
use tree_sitter::{Node, Point};

use crate::error::ParsingError;

/// Domain span of `node`. Columns are byte offsets within the line, as Tree-sitter reports them.
///
/// Positions beyond `u32::MAX` saturate; no real file reaches them.
///
/// # Errors
/// A model error if the span is inverted, which Tree-sitter never produces.
pub fn span_of(node: Node<'_>) -> Result<Span, ParsingError> {
    Ok(Span::new(
        position(node.start_position()),
        position(node.end_position()),
    )?)
}

fn position(point: Point) -> Position {
    Position {
        line: saturate(point.row),
        column: saturate(point.column),
    }
}

fn saturate(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// Text of `node`, or an empty string if it is not valid UTF-8 (it is, as the source is a `str`).
#[must_use]
pub fn text_of<'s>(node: Node<'_>, source: &'s str) -> &'s str {
    source.get(node.byte_range()).unwrap_or_default()
}
