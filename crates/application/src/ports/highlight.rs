//! Colouring the code of the source column (adapter: `codeskimr_parsing`).

use codeskimr_domain::RelativePath;

use crate::consts::{
    TOKEN_ATTRIBUTE, TOKEN_COMMENT, TOKEN_FUNCTION, TOKEN_KEYWORD, TOKEN_NUMBER, TOKEN_STRING,
    TOKEN_TYPE,
};

/// What a piece of code is, as far as the colours of the UI tell it apart. A language pack names
/// more in `highlights.scm`; whatever has no class here stays plain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenClass {
    /// A comment.
    Comment,
    /// A string or character literal, and its escapes.
    String,
    /// A number, or a constant such as `true` or `None`.
    Number,
    /// A keyword, or the object itself (`self`, `this`).
    Keyword,
    /// The name of a type, a tag.
    Type,
    /// The name of a function, a call or a macro.
    Function,
    /// An attribute, an annotation or a label.
    Attribute,
}

impl TokenClass {
    /// The word for the class on the wire.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Comment => TOKEN_COMMENT,
            Self::String => TOKEN_STRING,
            Self::Number => TOKEN_NUMBER,
            Self::Keyword => TOKEN_KEYWORD,
            Self::Type => TOKEN_TYPE,
            Self::Function => TOKEN_FUNCTION,
            Self::Attribute => TOKEN_ATTRIBUTE,
        }
    }
}

/// A run of code of one class inside one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSpan {
    /// Index of the line, counted from 0.
    pub line: u32,
    /// First byte of the run in that line.
    pub start: u32,
    /// Position right after the last byte of the run.
    pub end: u32,
    /// What the run is.
    pub class: TokenClass,
}

/// Finds the syntax of lines of code. Whoever calls this has read the lines already.
pub trait SyntaxHighlighter: Send + Sync {
    /// The runs of `lines` (the text of a file, or of a part of it) that have a class, in the
    /// order of the text, without overlaps. The language comes from the name of `path`. A language
    /// that has no `highlights.scm`, or a pack that is broken, gives no runs: the code is shown
    /// plain, never as a failure.
    fn highlight(&self, path: &RelativePath, lines: &[String]) -> Vec<TokenSpan>;
}
