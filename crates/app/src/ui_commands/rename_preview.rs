//! The command behind "Umbenennen prüfen…": what a new name for one symbol would hit, with
//! nothing changed (ADR 0110). It asks the same function as the MCP tool and the console command,
//! so the three answer alike.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, NameMentions, ProjectRepository, SourceText, SymbolKey, SymbolLabel, UsageIndex,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::rename::{self, RenameAsk, RenamePorts, RenamePreview};
use codeskimr_application::rename_rows::{
    ConflictRow, LayerRow, PlaceRow, PreviewRows, SiteRow, TextRow,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{
    RENAME_PREVIEW, RENAME_PREVIEW_DESCRIPTION, RENAME_PREVIEW_LIMIT, RENAME_PREVIEW_TITLE,
};

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

/// The answer of `rename.preview`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewOut {
    /// Id of the snapshot the answer is from.
    pub snapshot: u64,
    /// The symbol that would be renamed.
    pub symbol: PlaceRow,
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
    pub layers: Vec<LayerRow>,
    /// The first places.
    pub sites: Vec<SiteRow>,
    /// Symbols that already have the new name.
    pub conflicts: Vec<ConflictRow>,
    /// Whether the project folder was searched for text.
    pub text_searched: bool,
    /// All lines with the old name as text, also those the list cut.
    pub text_total: usize,
    /// Files the text search could not read.
    pub text_skipped: usize,
    /// The first of those lines.
    pub text_hits: Vec<TextRow>,
}

/// What the preview reads.
pub struct RenamePreviewPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) usages: Arc<dyn UsageIndex>,
    pub(super) mentions: Arc<dyn NameMentions>,
    pub(super) source: Arc<dyn SourceText>,
}

/// Shows what a new name for a symbol would hit.
pub struct RenamePreviewCommand {
    pub(super) ports: Arc<RenamePreviewPorts>,
}

fn answer(snapshot: u64, found: &RenamePreview) -> PreviewOut {
    let rows = PreviewRows::of(found);
    PreviewOut {
        snapshot,
        symbol: PlaceRow::from(&SymbolLabel {
            id: found.symbol.symbol.id,
            qualified_name: found.symbol.symbol.qualified_name.clone(),
            kind: found.symbol.symbol.kind,
        }),
        old_name: found.symbol.symbol.name.as_str().to_owned(),
        new_name: found.new_name.clone(),
        public_interface: found.public_interface,
        preliminary: found.verified < found.sites_total,
        sites_total: found.sites_total,
        verified: found.verified,
        files: found.files,
        layers: rows.layers,
        sites: rows.sites,
        conflicts: rows.conflicts,
        text_searched: found.text.searched,
        text_total: found.text.total,
        text_skipped: found.text.skipped,
        text_hits: rows.texts,
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
                source: ports.source.as_ref(),
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
        Ok(answer(snapshot.get(), &found))
    }
}
