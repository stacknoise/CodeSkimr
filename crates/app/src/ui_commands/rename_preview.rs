//! The command behind "Umbenennen prüfen…": what a new name for one symbol would hit, with
//! nothing changed (ADR 0110). It asks the same function as the MCP tool and the console command,
//! so the three answer alike.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, NameMentions, ProjectRepository, SymbolKey, SymbolLabel, UsageIndex,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::rename::{
    self, Conflict, RenameAsk, RenamePorts, RenamePreview, Site, TextHit,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Confidence, Permission, Project, ProjectId, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{
    COLUMN_OFFSET, LINE_OFFSET, RENAME_PREVIEW, RENAME_PREVIEW_DESCRIPTION, RENAME_PREVIEW_LIMIT,
    RENAME_PREVIEW_TITLE,
};
use super::usages::{Place, place};

/// The input of `rename.preview`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PreviewInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the symbol.
    pub symbol: u64,
    /// The proposed name.
    pub new_name: String,
}

/// One place of the code that would change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SiteOut {
    /// `declaration` or `usage`.
    pub kind: &'static str,
    /// File of the place.
    pub path: String,
    /// Line, from 1.
    pub line: u32,
    /// Column, from 1.
    pub column: u32,
    /// `certain` or `probable`; absent for a declaration.
    pub confidence: Option<&'static str>,
    /// Whether level 2 verified the place.
    pub verified: bool,
    /// The symbol that holds the place.
    pub from: Option<Place>,
    /// The layer that owns that symbol.
    pub layer: Option<String>,
}

/// A symbol that already has the new name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConflictOut {
    /// The symbol.
    pub symbol: Place,
    /// File that declares it.
    pub path: String,
    /// Its signature as written.
    pub signature: Option<String>,
    /// Whether the signature is the same but for the name (a clash, not an overload).
    pub same_signature: bool,
}

/// One line that holds the old name as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextOut {
    /// File of the line.
    pub path: String,
    /// Line, from 1.
    pub line: u32,
    /// The line as written, trimmed.
    pub text: String,
    /// `documentation`, `configuration`, `string`, `nameof`, `comment` or `code`.
    pub kind: &'static str,
}

/// How many places lie in one layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerOut {
    /// Name of the layer.
    pub layer: String,
    /// Number of places.
    pub count: usize,
}

/// The answer of `rename.preview`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewOut {
    /// Id of the snapshot the answer is from.
    pub snapshot: u64,
    /// The symbol that would be renamed.
    pub symbol: Place,
    /// The name it has.
    pub old_name: String,
    /// The proposed name.
    pub new_name: String,
    /// Whether code outside the project may use the symbol.
    pub public_interface: bool,
    /// Whether a listed place is not verified by level 2.
    pub preliminary: bool,
    /// All places, also those the list cut.
    pub sites_total: usize,
    /// Of them, those that level 2 verified.
    pub verified: usize,
    /// Files that hold places.
    pub files: usize,
    /// The places per layer.
    pub layers: Vec<LayerOut>,
    /// The first places.
    pub sites: Vec<SiteOut>,
    /// Symbols that already have the new name.
    pub conflicts: Vec<ConflictOut>,
    /// Whether the project folder was searched for text.
    pub text_searched: bool,
    /// All lines with the old name as text, also those the list cut.
    pub text_total: usize,
    /// Files the text search could not read.
    pub text_skipped: usize,
    /// The first of those lines.
    pub text_hits: Vec<TextOut>,
}

/// What the preview reads.
pub struct RenamePreviewPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) usages: Arc<dyn UsageIndex>,
    pub(super) mentions: Arc<dyn NameMentions>,
}

/// Shows what a new name for a symbol would hit.
pub struct RenamePreviewCommand {
    pub(super) ports: Arc<RenamePreviewPorts>,
}

fn site_out(project: &Project, site: &Site) -> SiteOut {
    SiteOut {
        path: site.path.clone(),
        layer: site.layer.clone(),
        verified: site.verified,
        kind: site.kind.label(),
        from: site.container.as_ref().map(|label| place(project, label)),
        line: site.line + LINE_OFFSET,
        column: site.column + COLUMN_OFFSET,
        confidence: site.confidence.map(Confidence::label),
    }
}

fn conflict_out(project: &Project, conflict: &Conflict) -> ConflictOut {
    ConflictOut {
        symbol: place(project, &conflict.symbol),
        path: conflict.path.clone(),
        signature: conflict.signature.clone(),
        same_signature: conflict.same_signature,
    }
}

fn label_of(found: &RenamePreview) -> SymbolLabel {
    SymbolLabel {
        id: found.symbol.symbol.id,
        qualified_name: found.symbol.symbol.qualified_name.clone(),
        kind: found.symbol.symbol.kind,
    }
}

fn text_out(hit: &TextHit) -> TextOut {
    TextOut {
        path: hit.path.clone(),
        line: hit.line + LINE_OFFSET,
        text: hit.text.clone(),
        kind: hit.kind.label(),
    }
}

fn answer(project: &Project, snapshot: u64, found: &RenamePreview) -> PreviewOut {
    PreviewOut {
        snapshot,
        symbol: place(project, &label_of(found)),
        old_name: found.symbol.symbol.name.as_str().to_owned(),
        new_name: found.new_name.clone(),
        public_interface: found.public_interface,
        preliminary: found.verified < found.sites_total,
        sites_total: found.sites_total,
        verified: found.verified,
        files: found.files,
        layers: found
            .layers
            .iter()
            .map(|count| LayerOut {
                layer: count.layer.clone(),
                count: count.count,
            })
            .collect(),
        sites: found
            .sites
            .iter()
            .map(|site| site_out(project, site))
            .collect(),
        conflicts: found
            .conflicts
            .iter()
            .map(|conflict| conflict_out(project, conflict))
            .collect(),
        text_searched: found.text.searched,
        text_total: found.text.total,
        text_skipped: found.text.skipped,
        text_hits: found.text.hits.iter().map(text_out).collect(),
    }
}

impl Command for RenamePreviewCommand {
    type Input = PreviewInput;
    type Output = PreviewOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(RENAME_PREVIEW),
        title_key: RENAME_PREVIEW_TITLE,
        description_key: RENAME_PREVIEW_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    /// # Errors
    /// `AUTH-401` without `read`, `API-101` for a new name that is no name or is the old one,
    /// `PRJ-201`, `MCP-203` and `MCP-202` for the project and the symbol, `SRC-201` if the
    /// folder cannot be searched, otherwise the failure of a port.
    fn execute(&self, request: Request<'_, Self>) -> Result<PreviewOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let input = request.input;
        let id = ProjectId::new(input.project);
        let project = ports.projects.project(id)?;
        let snapshot = ports
            .index
            .latest_snapshot(id)?
            .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))?
            .id;
        let symbol = ports
            .index
            .symbol(snapshot, &SymbolKey::Id(SymbolId::new(input.symbol)))?
            .ok_or_else(|| CodedError::new(ErrorCode::SymbolNotFound))?;
        let new_name = input.new_name.trim();
        if !rename::is_valid_name(new_name) || new_name == symbol.symbol.name.as_str() {
            return Err(CodedError::new(ErrorCode::RequestInvalid));
        }
        let root = rename::searchable_root(*request.principal, &project.source);
        let found = rename::preview(
            &RenamePorts {
                index: ports.index.as_ref(),
                usages: ports.usages.as_ref(),
                mentions: ports.mentions.as_ref(),
            },
            &RenameAsk {
                project: &project,
                snapshot,
                symbol: &symbol,
                new_name,
                text_root: root.as_ref(),
                limit: RENAME_PREVIEW_LIMIT,
            },
        )?;
        Ok(answer(&project, snapshot.get(), &found))
    }
}
