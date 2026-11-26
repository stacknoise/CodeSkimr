//! The command that exports a project as one self-contained HTML page (blueprint 10.2, 15).
//!
//! The page is the built UI with the answers of the read commands embedded: the tree of the
//! explorer, every symbol with its members, the usages of every symbol and the dependency map. It
//! needs no server. The export holds no source text, so a page that is passed on does not carry code
//! by accident. It dumps the whole index of a project in one go, so it needs `read_source` like
//! the other bulk readouts, and the HTTP route asks for the installation token (ADR 0035).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ids;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandExt, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, ProjectId, Snapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::consts::{
    EXPORT_BUNDLE_DIR, EXPORT_BUNDLE_FILE, EXPORT_DATA_PLACEHOLDER, EXPORT_DEPTH,
    EXPORT_DESCRIPTION, EXPORT_ESCAPE, EXPORT_ESCAPED_CHAR, EXPORT_FORMAT, EXPORT_FROM,
    EXPORT_HITS, EXPORT_HTML, EXPORT_ID, EXPORT_INCOMING, EXPORT_OUTGOING, EXPORT_PATH_SEPARATOR,
    EXPORT_TITLE, EXPORT_TO,
};
use super::explorer::{ExplorerOutline, Outline, OutlineInput};
use super::graph::{GraphInput, GraphLayers, GraphOut, Level};
use super::symbol::{ExplorerSymbol, SymbolInput, SymbolOut};
use super::usages::{Place, Summary, UsagesInput, UsagesOut, UsagesQuery, Way};

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
    /// Most references a project may have (`export.max_references`, ADR 0090).
    pub(super) max_references: u32,
}

/// Exports the newest scan of a project as one HTML page.
pub struct HtmlExport {
    parts: ExportParts,
    bundle_dir: Option<PathBuf>,
    embedded: Embedded,
    running: Arc<Mutex<BTreeSet<u64>>>,
}

/// Holds the place of a running export of one project and gives it back when it ends, however it
/// ends.
pub(super) struct Running {
    set: Arc<Mutex<BTreeSet<u64>>>,
    project: u64,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.set
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.project);
    }
}

/// The files of the UI that are part of the program: path below the UI folder and content.
pub(super) type Embedded = &'static [(&'static str, &'static [u8])];

/// The usages of one symbol: the walk, and the direct usages without tests for the switch of the
/// screen that hides them. The hits name their places by id; `Data::places` holds the places, so
/// a long qualified name is written once however often it is used.
#[derive(Debug, Serialize)]
struct Usages {
    answer: Value,
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
    /// The number of the scan inside the project, shown in the banner.
    scan_number: u64,
    project: Project,
    outline: Outline,
    symbols: BTreeMap<u64, SymbolOut>,
    places: BTreeMap<u64, Place>,
    usages: BTreeMap<u64, Usages>,
    graph: GraphOut,
}

/// Replaces the places `from` and `to` of one hit by their ids.
fn name_ends_by_id(hit: &mut Value) {
    for end in [EXPORT_FROM, EXPORT_TO] {
        if let Some(place) = hit.get_mut(end) {
            *place = place.get(EXPORT_ID).cloned().unwrap_or(Value::Null);
        }
    }
}

/// The answer as JSON in which every hit names `from` and `to` by the id of the place; the places
/// themselves go into `places`.
///
/// # Errors
/// `UnexpectedInternalError` if the answer cannot be written as JSON.
fn pack(answer: &UsagesOut, places: &mut BTreeMap<u64, Place>) -> Result<Value, CodedError> {
    let mut value = serde_json::to_value(answer)
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    for walk in [answer.incoming.as_ref(), answer.outgoing.as_ref()]
        .into_iter()
        .flatten()
    {
        for hit in &walk.hits {
            places
                .entry(hit.from.id)
                .or_insert_with(|| hit.from.clone());
            places.entry(hit.to.id).or_insert_with(|| hit.to.clone());
        }
    }
    for key in [EXPORT_INCOMING, EXPORT_OUTGOING] {
        if let Some(hits) = value
            .get_mut(key)
            .and_then(|walk| walk.get_mut(EXPORT_HITS))
            .and_then(Value::as_array_mut)
        {
            hits.iter_mut().for_each(name_ends_by_id);
        }
    }
    Ok(value)
}

/// The export page: from the UI folder if there is one, else from the files in the program.
pub(super) fn page_from(dir: Option<&Path>, embedded: Embedded) -> Result<String, CodedError> {
    let missing = || CodedError::new(ErrorCode::ExportBundleMissing);
    let Some(dir) = dir else {
        let name = [EXPORT_BUNDLE_DIR, EXPORT_BUNDLE_FILE].join(EXPORT_PATH_SEPARATOR);
        let (_, bytes) = embedded
            .iter()
            .find(|(path, _)| *path == name)
            .ok_or_else(missing)?;
        return String::from_utf8(bytes.to_vec()).map_err(|_| missing());
    };
    let file = dir.join(EXPORT_BUNDLE_DIR).join(EXPORT_BUNDLE_FILE);
    fs::read_to_string(file).map_err(|_| missing())
}

impl HtmlExport {
    /// An export that reads through `parts` and takes the page from the UI folder `bundle_dir`,
    /// like the server an existing folder wins. Without a folder the page comes from the files
    /// `embedded` in the program; without both every export fails with `EXP-201`.
    #[must_use]
    pub fn new(parts: ExportParts, bundle_dir: Option<PathBuf>, embedded: Embedded) -> Self {
        Self {
            parts,
            bundle_dir,
            embedded,
            running: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    /// Takes the place of the export of `project`; `EXP-501` if another export of it runs.
    pub(super) fn begin(&self, project: u64) -> Result<Running, CodedError> {
        let taken = !self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(project);
        if taken {
            return Err(CodedError::new(ErrorCode::ExportRunning));
        }
        Ok(Running {
            set: Arc::clone(&self.running),
            project,
        })
    }

    /// `EXP-502` if the newest scan has more references than the limit.
    fn check_size(&self, project: u64) -> Result<(), CodedError> {
        let (info, _) = self.project(ProjectId::new(project))?;
        if info.references > self.parts.max_references {
            return Err(CodedError::new(ErrorCode::ExportTooLarge));
        }
        Ok(())
    }

    fn page(&self) -> Result<String, CodedError> {
        page_from(self.bundle_dir.as_deref(), self.embedded)
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
            let symbol = self.parts.symbol.run(&principal, input)?;
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
        places: &mut BTreeMap<u64, Place>,
    ) -> Result<Usages, CodedError> {
        let input = |include_tests| UsagesInput {
            project,
            symbol,
            way: Way::Both,
            depth: EXPORT_DEPTH,
            include_tests,
        };
        let answer = self.parts.usages.run(&principal, input(true))?;
        let summary_without_tests = self.parts.usages.direct_summary(principal, input(false))?;
        Ok(Usages {
            answer: pack(&answer, places)?,
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
        let outline = self
            .parts
            .outline
            .run(&principal, OutlineInput { project })?;
        let symbols = self.symbols(principal, project, &outline)?;
        let mut places = BTreeMap::new();
        let mut usages = BTreeMap::new();
        for id in symbols.keys() {
            usages.insert(*id, self.usages_of(principal, project, *id, &mut places)?);
        }
        let graph = self.parts.graph.run(
            &principal,
            GraphInput {
                project,
                level: Level::Layers,
                depth: 0,
                focus: None,
                namespace: None,
            },
        )?;
        Ok(Data {
            format: EXPORT_FORMAT,
            snapshot: snapshot.id.get(),
            scan_number: ids::snapshot_number(snapshot.id),
            project: info,
            outline,
            symbols,
            places,
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
        permission: Permission::ReadSource,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ExportOut, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let _running = self.begin(request.input.project)?;
        let page = self.page()?;
        self.check_size(request.input.project)?;
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
    fn the_place_of_a_running_export_is_given_back_when_it_ends() {
        let set = Arc::new(Mutex::new(BTreeSet::from([7_u64])));
        let running = Running {
            set: Arc::clone(&set),
            project: 7,
        };
        assert!(set.lock().unwrap().contains(&7));
        drop(running);
        assert!(set.lock().unwrap().is_empty());
    }

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
