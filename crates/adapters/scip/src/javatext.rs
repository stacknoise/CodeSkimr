//! What the text of a Java file adds to an index of scip-java (issue #259, `docs/level2-import.md`
//! point 14): the import lines, which are no uses; the constructors with their parameters, the
//! place where the compiler puts the call of the constructor of the superclass that the source does
//! not write, and the creations of anonymous subclasses. All of it is read from the text with a
//! small scanner; where the text is not what the scanner expects it gives up and the index stays
//! as it is.

use std::cell::Cell;
use std::collections::HashSet;

use codeskimr_domain::{Position, Span};

use crate::consts::{
    BLOCK_COMMENT_CLOSE, BLOCK_COMMENT_OPEN, CLOSE_ANGLE, CLOSE_PAREN, COMMA,
    CONSTRUCTOR_CALL_KEYWORDS, CONSTRUCTOR_NAMES, GROUP_CLOSERS, GROUP_OPENERS, IMPORT_KEYWORD,
    JAVA_EXTENSION, JAVA_IDENTIFIER_EXTRA, LINE_COMMENT, LITERAL_ESCAPE, LITERAL_QUOTES,
    MAX_SCAN_CHARS, NEW_KEYWORD, NEWLINE, OPEN_BRACE, OPEN_GENERIC, OPEN_PAREN, PATH_DOT,
    QUALIFIED_SUPER, SCAN_BUDGET_FACTOR, SEMICOLON, SLASH, TEXT_BLOCK_MARK, THROWS_KEYWORD,
    VARARGS_MARK,
};
use crate::model::{AnonymousCreation, Arity, ConstructorSite, ScipDocument, SymbolRef, TextFacts};
use crate::symbol::{DescriptorKind, is_local, last_descriptor};

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || JAVA_IDENTIFIER_EXTRA.contains(&c)
}

/// The lines of a file and what all scans of the file may still read together: a file that
/// needs more than `SCAN_BUDGET_FACTOR` times its size is not read on, and nothing is marked in it.
struct Text<'a> {
    lines: &'a [String],
    budget: Cell<usize>,
}

impl<'a> Text<'a> {
    fn new(lines: &'a [String]) -> Self {
        let size: usize = lines.iter().map(String::len).sum();
        let budget = size.saturating_mul(SCAN_BUDGET_FACTOR).max(MAX_SCAN_CHARS);
        Self {
            lines,
            budget: Cell::new(budget),
        }
    }

    fn exhausted(&self) -> bool {
        self.budget.get() == 0
    }
}

/// A reading position in the lines of a file, with a budget of characters.
struct Cursor<'a> {
    text: &'a Text<'a>,
    line: usize,
    col: usize,
    left: usize,
}

impl<'a> Cursor<'a> {
    fn at(text: &'a Text<'a>, at: Position) -> Option<Self> {
        let line = usize::try_from(at.line).ok()?;
        let col = usize::try_from(at.column).ok()?;
        text.lines.get(line)?.get(col..)?;
        Some(Self {
            text,
            line,
            col,
            left: MAX_SCAN_CHARS,
        })
    }

    fn rest(&self) -> &str {
        self.text
            .lines
            .get(self.line)
            .and_then(|line| line.get(self.col..))
            .unwrap_or_default()
    }

    /// The next character; the end of a line reads as a line feed, the end of the file as nothing.
    fn peek(&self) -> Option<char> {
        if self.left == 0 || self.text.exhausted() {
            return None;
        }
        match self.rest().chars().next() {
            Some(c) => Some(c),
            None => (self.line.saturating_add(1) < self.text.lines.len()).then_some(NEWLINE),
        }
    }

    fn bump(&mut self) {
        if let Some(c) = self.rest().chars().next() {
            self.col = self.col.saturating_add(c.len_utf8());
        } else {
            self.line = self.line.saturating_add(1);
            self.col = 0;
        }
        self.spend();
    }

    /// Takes one character off the budget of this scan and of the file.
    fn spend(&mut self) {
        self.left = self.left.saturating_sub(1);
        let file = &self.text.budget;
        file.set(file.get().saturating_sub(1));
    }

    fn position(&self) -> Option<Position> {
        Some(Position {
            line: u32::try_from(self.line).ok()?,
            column: u32::try_from(self.col).ok()?,
        })
    }

    fn at_comment(&self) -> bool {
        let rest = self.rest();
        rest.starts_with(LINE_COMMENT) || rest.starts_with(BLOCK_COMMENT_OPEN)
    }

    /// Whether the text goes on with `word` as a whole word.
    fn at_word(&self, word: &str) -> bool {
        self.rest()
            .strip_prefix(word)
            .is_some_and(|after| !after.chars().next().is_some_and(is_identifier_char))
    }

    /// Skips white space and comments.
    fn skip_blank(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
            } else if c == SLASH && self.rest().starts_with(LINE_COMMENT) {
                self.line = self.line.saturating_add(1);
                self.col = 0;
                self.spend();
            } else if c == SLASH && self.rest().starts_with(BLOCK_COMMENT_OPEN) {
                self.skip_block_comment();
            } else {
                return;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        while self.peek().is_some() && !self.rest().starts_with(BLOCK_COMMENT_CLOSE) {
            self.bump();
        }
        for _ in BLOCK_COMMENT_CLOSE.chars() {
            self.bump();
        }
    }

    /// Skips a string or character literal; one that does not end on its line is left at the line
    /// end.
    fn skip_literal(&mut self, quote: char) {
        if quote == LITERAL_QUOTES[0] && self.rest().starts_with(TEXT_BLOCK_MARK) {
            self.skip_text_block();
            return;
        }
        self.bump();
        while let Some(c) = self.peek() {
            if c == NEWLINE {
                return;
            }
            self.bump();
            if c == LITERAL_ESCAPE {
                self.bump();
            } else if c == quote {
                return;
            }
        }
    }

    /// Skips a text block (`"""` up to the next `"""` that is not escaped), over lines.
    fn skip_text_block(&mut self) {
        for _ in TEXT_BLOCK_MARK.chars() {
            self.bump();
        }
        while let Some(c) = self.peek() {
            if self.rest().starts_with(TEXT_BLOCK_MARK) {
                TEXT_BLOCK_MARK.chars().for_each(|_| self.bump());
                return;
            }
            self.bump();
            if c == LITERAL_ESCAPE {
                self.bump();
            }
        }
    }

    /// Skips a group that may be missing: false only if it starts but does not end.
    fn skip_optional_group(&mut self, open: char, close: char) -> bool {
        self.skip_blank();
        self.peek() != Some(open) || self.skip_group(open, close)
    }

    /// Skips a group from its `open` character to the matching `close`; false if there is none.
    fn skip_group(&mut self, open: char, close: char) -> bool {
        if self.peek() != Some(open) {
            return false;
        }
        let mut depth = usize::default();
        while let Some(c) = self.peek() {
            if LITERAL_QUOTES.contains(&c) {
                self.skip_literal(c);
                continue;
            }
            if self.at_comment() {
                self.skip_blank();
                continue;
            }
            self.bump();
            if c == open {
                depth = depth.saturating_add(1);
            } else if c == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return true;
                }
            }
        }
        false
    }
}

/// The parameters of a constructor, counted while they are read.
#[derive(Default)]
struct Parameters {
    depth: usize,
    commas: usize,
    content: bool,
    varargs: bool,
}

impl Parameters {
    /// Takes one character in; `None` if a group is closed that was not opened.
    fn take(&mut self, c: char) -> Option<()> {
        if GROUP_OPENERS.contains(&c) {
            self.depth = self.depth.saturating_add(1);
        } else if GROUP_CLOSERS.contains(&c) {
            self.depth = self.depth.checked_sub(1)?;
        } else if c == COMMA && self.depth == 0 {
            self.commas = self.commas.saturating_add(1);
        }
        self.content = true;
        Some(())
    }

    const fn arity(&self) -> Arity {
        if !self.content {
            Arity::Zero
        } else if self.commas == 0 {
            Arity::One {
                varargs: self.varargs,
            }
        } else {
            Arity::Several
        }
    }
}

impl Cursor<'_> {
    /// Reads the parameter list that starts at the cursor.
    fn parameters(&mut self) -> Option<Arity> {
        if self.peek() != Some(OPEN_PAREN) {
            return None;
        }
        self.bump();
        let mut state = Parameters::default();
        loop {
            self.skip_blank();
            let c = self.peek()?;
            if c == CLOSE_PAREN && state.depth == 0 {
                self.bump();
                return Some(state.arity());
            }
            if self.rest().starts_with(VARARGS_MARK) {
                state.varargs = true;
                state.content = true;
                VARARGS_MARK.chars().for_each(|_| self.bump());
                continue;
            }
            state.take(c)?;
            if LITERAL_QUOTES.contains(&c) {
                self.skip_literal(c);
            } else {
                self.bump();
            }
        }
    }

    /// The opening brace of the body, after the parameters and an optional `throws` list.
    fn body_start(&mut self) -> Option<Position> {
        self.skip_blank();
        if self.at_word(THROWS_KEYWORD) {
            while self.peek()? != OPEN_BRACE {
                if self.peek()? == SEMICOLON {
                    return None;
                }
                self.bump();
            }
        }
        (self.peek()? == OPEN_BRACE).then(|| self.position())?
    }

    /// Whether the body that opens at the cursor starts with a call of `this` or `super`, also
    /// the qualified `outer.super(...)`.
    fn starts_with_constructor_call(&mut self) -> bool {
        self.bump();
        self.skip_blank();
        let mut path = String::new();
        while let Some(c) = self.peek() {
            if !is_identifier_char(c) && c != PATH_DOT {
                break;
            }
            path.push(c);
            self.bump();
        }
        let is_call =
            CONSTRUCTOR_CALL_KEYWORDS.contains(&path.as_str()) || path.ends_with(QUALIFIED_SUPER);
        self.skip_blank();
        is_call && self.peek() == Some(OPEN_PAREN)
    }
}

/// A constructor of the document with what its text tells; `None` where the text is not as
/// expected.
fn site_of(text: &Text<'_>, symbol: SymbolRef, name: Span) -> Option<ConstructorSite> {
    let mut cursor = Cursor::at(text, name.end())?;
    cursor.skip_blank();
    if cursor.peek() != Some(OPEN_PAREN) {
        return Some(ConstructorSite {
            symbol,
            declared: false,
            arity: Arity::Zero,
            call_at: name,
            explicit_call: false,
        });
    }
    let arity = cursor.parameters()?;
    let open = cursor.body_start()?;
    let explicit_call = Cursor::at(text, open)?.starts_with_constructor_call();
    Some(ConstructorSite {
        symbol,
        declared: true,
        arity,
        call_at: one_char(open)?,
        explicit_call,
    })
}

/// The opening brace of the anonymous class that starts after the name of a type that ends at
/// `end`: `new Name<..>(..) {`.
fn anonymous_body(text: &Text<'_>, end: Position) -> Option<Span> {
    let mut cursor = Cursor::at(text, end)?;
    if !cursor.skip_optional_group(OPEN_GENERIC, CLOSE_ANGLE) {
        return None;
    }
    cursor.skip_blank();
    if !cursor.skip_group(OPEN_PAREN, CLOSE_PAREN) {
        return None;
    }
    cursor.skip_blank();
    let open = (cursor.peek()? == OPEN_BRACE).then(|| cursor.position())??;
    one_char(open)
}

/// Whether the name of an enum constant that ends at `end` is followed by a body: `NAME(..) {` or
/// `NAME {`.
fn enum_body_follows(text: &Text<'_>, end: Position) -> bool {
    let Some(mut cursor) = Cursor::at(text, end) else {
        return false;
    };
    if !cursor.skip_optional_group(OPEN_PAREN, CLOSE_PAREN) {
        return false;
    }
    cursor.skip_blank();
    cursor.peek() == Some(OPEN_BRACE)
}

/// Whether the symbol is a term (a field or an enum constant).
fn is_term(symbols: &[String], at: SymbolRef) -> bool {
    last_kind(symbols, at) == Some(DescriptorKind::Term)
}

fn is_constructor(symbols: &[String], at: SymbolRef) -> bool {
    symbol_text(symbols, at)
        .and_then(last_descriptor)
        .is_some_and(|last| {
            last.kind == DescriptorKind::Method && CONSTRUCTOR_NAMES.contains(&last.name.as_str())
        })
}

/// The text of a global symbol.
fn symbol_text(symbols: &[String], at: SymbolRef) -> Option<&str> {
    usize::try_from(at.0)
        .ok()
        .and_then(|position| symbols.get(position))
        .map(String::as_str)
        .filter(|text| !is_local(text))
}

/// What the last descriptor of a global symbol describes.
fn last_kind(symbols: &[String], at: SymbolRef) -> Option<DescriptorKind> {
    symbol_text(symbols, at)
        .and_then(last_descriptor)
        .map(|last| last.kind)
}

/// The span of the one character at `at`.
fn one_char(at: Position) -> Option<Span> {
    let after = Position {
        line: at.line,
        column: at.column.saturating_add(1),
    };
    Span::new(at, after).ok()
}

/// Whether the word `new` stands right before `start`.
fn follows_new(text: &Text<'_>, start: Position) -> bool {
    let line = usize::try_from(start.line)
        .ok()
        .and_then(|at| text.lines.get(at));
    let before = line
        .zip(usize::try_from(start.column).ok())
        .and_then(|(line, column)| line.get(..column));
    before
        .map(str::trim_end)
        .and_then(|text| text.strip_suffix(NEW_KEYWORD))
        .is_some_and(|text| !text.chars().next_back().is_some_and(is_identifier_char))
}

fn is_type(symbols: &[String], at: SymbolRef) -> bool {
    last_kind(symbols, at) == Some(DescriptorKind::Type)
}

/// The lines of the import declarations: a line that starts with the keyword and the lines up to
/// the semicolon.
fn import_lines(lines: &[String]) -> HashSet<usize> {
    let mut found = HashSet::new();
    let mut inside = false;
    for (at, line) in lines.iter().enumerate() {
        let starts = line
            .trim_start()
            .strip_prefix(IMPORT_KEYWORD)
            .is_some_and(|after| after.chars().next().is_some_and(char::is_whitespace));
        inside |= starts;
        if inside {
            found.insert(at);
            inside = !line.contains(SEMICOLON);
        }
    }
    found
}

/// Whether the facts of the text are read for this document: a Java file whose columns are bytes.
pub(crate) fn wants_facts(document: &ScipDocument) -> bool {
    document.path.as_str().ends_with(JAVA_EXTENSION)
}

/// Reads the text facts of one document (its columns are bytes) and drops what is no use.
pub(crate) fn read_document(document: &mut ScipDocument, lines: &[String], symbols: &[String]) {
    let imports = import_lines(lines);
    document.occurrences.retain(|occurrence| {
        usize::try_from(occurrence.range.start().line).map_or(true, |line| !imports.contains(&line))
    });
    let text = Text::new(lines);
    let constructors = document
        .occurrences
        .iter()
        .filter(|o| o.is_definition() && is_constructor(symbols, o.symbol))
        .filter_map(|o| site_of(&text, o.symbol, o.range))
        .collect();
    let anonymous = document
        .occurrences
        .iter()
        .filter(|o| !o.is_definition() && follows_new(&text, o.range.start()))
        .filter(|o| is_type(symbols, o.symbol))
        .filter_map(|o| {
            Some(AnonymousCreation {
                symbol: o.symbol,
                range: o.range,
                body: anonymous_body(&text, o.range.end())?,
            })
        })
        .collect();
    let enum_bodies = document
        .occurrences
        .iter()
        .filter(|o| o.is_definition() && is_term(symbols, o.symbol))
        .filter(|o| enum_body_follows(&text, o.range.end()))
        .map(|o| o.range)
        .collect();
    // A file that used up its budget was read only in part: nothing is marked in it.
    document.facts = if text.exhausted() {
        TextFacts::default()
    } else {
        TextFacts {
            constructors,
            anonymous,
            enum_bodies,
        }
    };
}
