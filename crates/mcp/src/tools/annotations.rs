//! `list_annotations` (ADR 0068): the notes, tasks and critical marks of a project or a symbol. It
//! only reads and needs `read`; agents cannot write annotations.

use codeskimr_application::access;
use codeskimr_application::annotations::{AnnotationQuery, ContentTexts, annotations};
use codeskimr_domain::{Annotation, AnnotationKind, AnnotationTarget, Principal, Symbol};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, optional_label, to_json};
use crate::args::AnnotationsArgs;
use crate::consts::{ARG_KIND, ARG_STATUS, READ_TOOL_PERMISSION};
use crate::error::McpError;

/// One annotation, with only what an agent needs; a field that is empty is left out.
#[derive(Serialize)]
pub(super) struct Item {
    id: u64,
    /// `critical`, `note` or `task`.
    kind: &'static str,
    /// Kind of the symbol.
    symbol_kind: &'static str,
    /// Qualified name of the symbol.
    symbol: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
    /// Seconds since the Unix epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    /// Only present, and `true`, if the newest scan does not find the symbol.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    without_target: bool,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    total: u32,
    offset: u32,
    /// Pass it as `offset` to read the next page; absent on the last one.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_offset: Option<u32>,
    annotations: Vec<Item>,
}

pub(super) fn item(annotation: &Annotation) -> Item {
    let texts = ContentTexts::from(&annotation.content);
    Item {
        id: annotation.id.get(),
        kind: annotation.content.kind.label(),
        symbol_kind: annotation.target.kind.label(),
        symbol: annotation.target.name.as_str().to_owned(),
        parameters: annotation.target.parameters.clone(),
        title: texts.title,
        text: texts.text,
        status: texts.status,
        due: texts.due,
        assignee: texts.assignee,
        without_target: annotation.without_target,
    }
}

/// The target that selects the annotations of every overload of `symbol`.
pub(super) fn target_of(symbol: Symbol) -> AnnotationTarget {
    AnnotationTarget {
        kind: symbol.kind,
        name: symbol.qualified_name,
        parameters: Vec::new(),
    }
}

impl Tools {
    /// The annotations of a project, or of one symbol, newest scan's view of them.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid argument, `MCP-202` if
    /// the symbol does not exist, `MCP-201` and `MCP-203` for the project, or the code of a
    /// failing port.
    pub fn list_annotations(
        &self,
        principal: &Principal,
        args: &AnnotationsArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let kind = optional_label::<AnnotationKind>(args.kind.as_deref(), ARG_KIND)?;
        let status = optional_label(args.status.as_deref(), ARG_STATUS)?;
        let scope = self.scope(args.project)?;
        let target = match &args.symbol {
            Some(text) => Some(target_of(self.find_symbol(&scope, text)?.symbol)),
            None => None,
        };
        let query = AnnotationQuery {
            project: scope.project.id,
            target,
            kind,
            status,
            without_target: None,
        };
        let page = annotations(
            self.annotation_store(),
            &query,
            self.page_request(args.offset, args.limit),
        )?;
        to_json(&Answer {
            snapshot: scope.info,
            total: page.total,
            offset: page.offset,
            next_offset: page.next_offset(),
            annotations: page.items.iter().map(item).collect(),
        })
    }
}
