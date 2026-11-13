//! Positions and spans inside a source file.

use crate::ModelError;

/// A zero-based line and column in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// Zero-based line.
    pub line: u32,
    /// Zero-based column.
    pub column: u32,
}

/// A range in a source file, from `start` up to `end`. `end` is never before `start`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    /// Creates a span.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvertedSpan`] if `end` is before `start`.
    pub fn new(start: Position, end: Position) -> Result<Self, ModelError> {
        if end < start {
            Err(ModelError::InvertedSpan)
        } else {
            Ok(Self { start, end })
        }
    }

    /// First position of the span.
    #[must_use]
    pub const fn start(&self) -> Position {
        self.start
    }

    /// Position right after the span.
    #[must_use]
    pub const fn end(&self) -> Position {
        self.end
    }

    /// Number of lines the span touches, at least one.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.end.line - self.start.line + 1
    }
}
