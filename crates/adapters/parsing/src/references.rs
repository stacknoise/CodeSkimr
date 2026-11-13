//! Extraction of unresolved name occurrences with the reference query of a language pack.

use codeskimr_application::ports::PendingReference;
use codeskimr_domain::{QualifiedName, ReferenceKind};
use tree_sitter::{Node, QueryCursor, StreamingIterator};

use crate::captures::Capture;
use crate::error::ParsingError;
use crate::location::{span_of, text_of};
use crate::symbols::{SymbolContext, SymbolTable};

/// Finds every occurrence the pack's `references.scm` marks. Occurrences outside any declaration
/// belong to the module symbol of the file. Packs without the query yield no occurrences.
///
/// # Errors
/// A model error if an occurrence has an invalid span, which Tree-sitter never produces.
pub fn extract(
    ctx: &SymbolContext<'_>,
    root: Node<'_>,
    table: &SymbolTable,
) -> Result<Vec<PendingReference>, ParsingError> {
    let Some(vocabulary) = &ctx.pack.references else {
        return Ok(Vec::new());
    };
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&vocabulary.query, root, ctx.source.as_bytes());
    let mut pending = Vec::new();
    while let Some(found) = matches.next() {
        let mut kind = None;
        let mut occurrence = None;
        let mut name = None;
        for capture in found.captures() {
            match vocabulary.role(capture.index) {
                Some(Capture::Reference(found_kind)) => {
                    kind = Some(found_kind);
                    occurrence = Some(capture.node);
                }
                Some(Capture::Name) => name = Some(capture.node),
                _ => {}
            }
        }
        if let (Some(kind), Some(node)) = (kind, name.or(occurrence)) {
            pending.extend(pending_reference(ctx, table, kind, node)?);
        }
    }
    Ok(pending)
}

fn pending_reference(
    ctx: &SymbolContext<'_>,
    table: &SymbolTable,
    kind: ReferenceKind,
    node: Node<'_>,
) -> Result<Option<PendingReference>, ParsingError> {
    let Some(from) = table.innermost(node.start_byte()) else {
        return Ok(None);
    };
    let Ok(target) = QualifiedName::new(text_of(node, ctx.source).trim()) else {
        return Ok(None);
    };
    Ok(Some(PendingReference {
        from: from.id,
        file: ctx.file,
        span: span_of(node)?,
        kind,
        target,
    }))
}
