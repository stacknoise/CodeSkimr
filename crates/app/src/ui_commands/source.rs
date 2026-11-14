//! The command behind the source column of the explorer: the lines of one declaration, read from
//! the project folder as it is now. It hands out source text and therefore needs `read_source`
//! and, like every command that changes something, the installation token (ADR 0035).

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::CodeSpan;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId, ProjectSource, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{
    EXPLORER_SOURCE, LINE_OFFSET, SOURCE_DESCRIPTION, SOURCE_MAX_BYTES, SOURCE_MAX_LINES,
    SOURCE_MAX_SPANS, SOURCE_TITLE,
};
use super::explorer::ExplorerPorts;

/// The input of `explorer.source`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SourceInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the symbol, a type or a member.
    pub symbol: u64,
}

/// A place in the code where a symbol is named: the text between `start` and `end` of the line
/// stands for `symbol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SpanOut {
    /// Index of the line within `lines`, counted from 0.
    pub line: u32,
    /// First UTF-16 unit of the name in that line, counted from 0 (what a browser counts).
    pub start: u32,
    /// Position right after the last unit of the name.
    pub end: u32,
    /// Id of the symbol that is named.
    pub symbol: u64,
}

/// The answer of `explorer.source`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceOut {
    /// Path of the declaring file.
    pub path: String,
    /// First line of the declaration, counted from 1.
    pub first_line: u32,
    /// Last line of the declaration, counted from 1.
    pub last_line: u32,
    /// Whether the code could be read: only projects that are a folder on this machine can.
    pub available: bool,
    /// The lines from `first_line` on, without line ends.
    pub lines: Vec<String>,
    /// Whether the declaration is longer than the lines returned.
    pub truncated: bool,
    /// The places in `lines` where a symbol is named, in the order of the text. Used for Alt+click
    /// on a name; a project whose scan holds no references has none.
    pub spans: Vec<SpanOut>,
}

/// Turns the places of the index (zero-based lines of the file, columns in bytes) into places in
/// the lines that were read (UTF-16 units). A place that does not fit its line is left out: the file
/// has changed since the scan.
fn span_outs(spans: &[CodeSpan], first_line: u32, lines: &[String]) -> Vec<SpanOut> {
    spans
        .iter()
        .filter_map(|found| {
            let line = found.span.start().line.checked_sub(first_line)?;
            let text = lines.get(usize::try_from(line).ok()?)?;
            let start = text.get(..usize::try_from(found.span.start().column).ok()?)?;
            let end = text.get(..usize::try_from(found.span.end().column).ok()?)?;
            Some(SpanOut {
                line,
                start: u32::try_from(start.encode_utf16().count()).ok()?,
                end: u32::try_from(end.encode_utf16().count()).ok()?,
                symbol: found.to.get(),
            })
        })
        .take(SOURCE_MAX_SPANS)
        .collect()
}

/// The lines of the declaration of one symbol.
pub struct ExplorerSource {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for ExplorerSource {
    type Input = SourceInput;
    type Output = SourceOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(EXPLORER_SOURCE),
        title_key: SOURCE_TITLE,
        description_key: SOURCE_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::ReadSource,
    };

    fn execute(&self, request: Request<'_, SourceInput>) -> Result<SourceOut, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let ports = &self.ports;
        let project = ports
            .projects
            .project(ProjectId::new(request.input.project))?;
        let snapshot = ports.newest(request.input.project)?.id;
        let detail = ports.lookup(snapshot, SymbolId::new(request.input.symbol))?;
        let first = detail.symbol.span.start().line + LINE_OFFSET;
        let last = detail.symbol.span.end().line + LINE_OFFSET;
        let mut out = SourceOut {
            path: detail.path.to_string(),
            first_line: first,
            last_line: last,
            available: false,
            lines: Vec::new(),
            truncated: false,
            spans: Vec::new(),
        };
        let ProjectSource::Folder(root) = &project.source else {
            return Ok(out);
        };
        let excerpt = ports.text.excerpt(
            root,
            &detail.path,
            (first, last),
            (SOURCE_MAX_LINES, SOURCE_MAX_BYTES),
        )?;
        out.available = true;
        let shown = u32::try_from(excerpt.lines.len()).unwrap_or(u32::MAX);
        let spans = ports.index.code_spans(
            snapshot,
            &detail.path,
            (
                first - LINE_OFFSET,
                (first - LINE_OFFSET).saturating_add(shown.saturating_sub(1)),
            ),
        )?;
        out.spans = span_outs(&spans, first - LINE_OFFSET, &excerpt.lines);
        out.lines = excerpt.lines;
        out.truncated = excerpt.truncated;
        Ok(out)
    }
}
