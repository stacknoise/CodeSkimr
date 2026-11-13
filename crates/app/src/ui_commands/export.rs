//! The command that exports a project as one self-contained HTML page (blueprint 10.2, 15).
//!
//! The page is the built UI with the answers of the read commands embedded: the tree of the
//! explorer, every symbol with its members, the usages of every symbol and the dependency map. It
//! needs no server. The export holds no source text: that needs `read_source`, and a page that is
//! passed on must not carry code by accident.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, ProjectId, Snapshot};
use serde::{Deserialize, Serialize};

use super::consts::{
    EXPORT_BUNDLE_DIR, EXPORT_BUNDLE_FILE, EXPORT_DATA_PLACEHOLDER, EXPORT_DEPTH,
    EXPORT_DESCRIPTION, EXPORT_DIRECT_DEPTH, EXPORT_ESCAPE, EXPORT_ESCAPED_CHAR, EXPORT_FORMAT,
    EXPORT_HTML, EXPORT_TITLE,
};
use super::explorer::{ExplorerOutline, Outline, OutlineInput};
use super::graph::{GraphInput, GraphLayers, GraphOut};
use super::symbol::{ExplorerSymbol, SymbolInput, SymbolOut};
use super::usages::{Summary, UsagesInput, UsagesOut, UsagesQuery, Way};

/// The input of `export.html`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ExportInput {
    /// Id of the project.
    pub project: u64,
}

/// The answer of `export.html`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportOut {
    /// The complete page.
    pub html: String,
    /// Number of symbols in it.
    pub symbols: usize,
}

/// The commands the export reads through.
pub struct ExportParts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) outline: ExplorerOutline,
    pub(super) symbol: ExplorerSymbol,
    pub(super) usages: UsagesQuery,
    pub(super) graph: GraphLayers,
}

/// Exports the newest scan of a project as one HTML page.
pub struct HtmlExport {
    parts: ExportParts,
    bundle_dir: Option<PathBuf>,
}

/// The usages of one symbol: the full walk, and the direct usages without tests for the switch of
/// the screen that hides them.
#[derive(Debug, Serialize)]
struct Usages {
    answer: UsagesOut,
    summary_without_tests: Summary,
}

/// What the page says about the project. It names no folder or address: the page is passed on.
#[derive(Debug, Serialize)]
struct Project {
    id: u64,
    name: String,
    commit: Option<String>,
    created: i64,
    mode: &'static str,
    files: u32,
    symbols: u32,
    references: u32,
}

/// What the page reads: everything the screens ask the server for.
#[derive(Debug, Serialize)]
struct Data {
    format: u32,
    snapshot: u64,
    project: Project,
    outline: Outline,
    symbols: BTreeMap<u64, SymbolOut>,
    usages: BTreeMap<u64, Usages>,
    graph: GraphOut,
}

impl HtmlExport {
    /// An export that reads through `parts` and takes the page from the UI folder `bundle_dir`;
    /// without a folder every export fails with `EXP-201`.
    #[must_use]
    pub const fn new(parts: ExportParts, bundle_dir: Option<PathBuf>) -> Self {
        Self { parts, bundle_dir }
    }

    fn page(&self) -> Result<String, CodedError> {
        let file = self
            .bundle_dir
            .as_ref()
            .map(|dir| dir.join(EXPORT_BUNDLE_DIR).join(EXPORT_BUNDLE_FILE))
            .ok_or_else(|| CodedError::new(ErrorCode::ExportBundleMissing))?;
        fs::read_to_string(file).map_err(|_| CodedError::new(ErrorCode::ExportBundleMissing))
    }

    fn symbols(
        &self,
        principal: Principal,
        project: u64,
        outline: &Outline,
    ) -> Result<BTreeMap<u64, SymbolOut>, CodedError> {
        let mut open: Vec<u64> = outline
            .layers
            .iter()
            .flat_map(|layer| &layer.namespaces)
            .flat_map(|namespace| &namespace.types)
            .map(|node| node.id)
            .collect();
        let mut seen = BTreeSet::new();
        let mut symbols = BTreeMap::new();
        while let Some(id) = open.pop() {
            if !seen.insert(id) {
                continue;
            }
            let input = SymbolInput {
                project,
                symbol: id,
            };
            let symbol = self.parts.symbol.execute(Request {
                principal: &principal,
                input,
            })?;
            open.extend(symbol.members.iter().map(|member| member.id));
            symbols.insert(id, symbol);
        }
        Ok(symbols)
    }

    fn usages_of(
        &self,
        principal: Principal,
        project: u64,
        symbol: u64,
    ) -> Result<Usages, CodedError> {
        let ask = |way, depth, include_tests| {
            let input = UsagesInput {
                project,
                symbol,
                way,
                depth,
                include_tests,
            };
            self.parts.usages.execute(Request {
                principal: &principal,
                input,
            })
        };
        let answer = ask(Way::Both, EXPORT_DEPTH, true)?;
        let summary_without_tests = ask(Way::Incoming, EXPORT_DIRECT_DEPTH, false)?.summary;
        Ok(Usages {
            answer,
            summary_without_tests,
        })
    }

    fn project(&self, id: ProjectId) -> Result<(Project, Snapshot), CodedError> {
        let project = self.parts.projects.project(id)?;
        let snapshot = self
            .parts
            .index
            .latest_snapshot(id)?
            .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))?;
        let info = Project {
            id: project.id.get(),
            name: project.name.to_string(),
            commit: snapshot.commit.as_ref().map(ToString::to_string),
            created: snapshot.created.0,
            mode: snapshot.mode.label(),
            files: snapshot.stats.files,
            symbols: snapshot.stats.symbols,
            references: snapshot.stats.references,
        };
        Ok((info, snapshot))
    }

    fn collect(&self, principal: Principal, project: u64) -> Result<Data, CodedError> {
        let (info, snapshot) = self.project(ProjectId::new(project))?;
        let outline = self.parts.outline.execute(Request {
            principal: &principal,
            input: OutlineInput { project },
        })?;
        let symbols = self.symbols(principal, project, &outline)?;
        let usages = symbols
            .keys()
            .map(|id| Ok((*id, self.usages_of(principal, project, *id)?)))
            .collect::<Result<BTreeMap<_, _>, CodedError>>()?;
        let graph = self.parts.graph.execute(Request {
            principal: &principal,
            input: GraphInput { project },
        })?;
        Ok(Data {
            format: EXPORT_FORMAT,
            snapshot: snapshot.id.get(),
            project: info,
            outline,
            symbols,
            usages,
            graph,
        })
    }
}

/// Puts the data into the page, in the one place the page keeps for it. The data is JSON inside
/// a script element, so `<` is written as an escape: no text of the project can end the element.
///
/// # Errors
/// `EXP-201` if the page has no such place or more than one.
fn embed(page: &str, data: &str) -> Result<String, CodedError> {
    if page.matches(EXPORT_DATA_PLACEHOLDER).count() != 1 {
        return Err(CodedError::new(ErrorCode::ExportBundleMissing));
    }
    let safe = data.replace(EXPORT_ESCAPED_CHAR, EXPORT_ESCAPE);
    Ok(page.replace(EXPORT_DATA_PLACEHOLDER, &safe))
}

impl Command for HtmlExport {
    type Input = ExportInput;
    type Output = ExportOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(EXPORT_HTML),
        title_key: EXPORT_TITLE,
        description_key: EXPORT_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, ExportInput>) -> Result<ExportOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let page = self.page()?;
        let data = self.collect(*request.principal, request.input.project)?;
        let json = serde_json::to_string(&data)
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
        Ok(ExportOut {
            html: embed(&page, &json)?,
            symbols: data.symbols.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use codeskimr_codes::Coded;

    use super::*;

    #[test]
    fn the_data_takes_the_place_the_page_keeps() {
        let page = format!("<script>{EXPORT_DATA_PLACEHOLDER}</script>");
        let html = embed(&page, "{\"a\":1}").unwrap();
        assert_eq!(html, "<script>{\"a\":1}</script>");
    }

    #[test]
    fn text_of_the_project_cannot_end_the_script_element() {
        let page = format!("<script>{EXPORT_DATA_PLACEHOLDER}</script>");
        let html = embed(&page, "{\"doc\":\"</script><b>\"}").unwrap();
        assert_eq!(html.matches("</script>").count(), 1);
        assert!(html.contains("\\u003c/script>\\u003cb>"));
    }

    #[test]
    fn a_page_without_a_place_for_the_data_is_refused() {
        let error = embed("<html></html>", "{}").unwrap_err();
        assert_eq!(error.code(), ErrorCode::ExportBundleMissing);
        let twice = format!("{EXPORT_DATA_PLACEHOLDER}{EXPORT_DATA_PLACEHOLDER}");
        assert_eq!(
            embed(&twice, "{}").unwrap_err().code(),
            ErrorCode::ExportBundleMissing
        );
    }
}
