//! `preview_rename`.

use codeskimr_application::access;
use codeskimr_application::rename::{self, RenameAsk, RenamePorts, RenamePreview};
use codeskimr_application::rename_rows::{ConflictRow, LayerRow, PreviewRows, SiteRow, TextRow};
use codeskimr_domain::Principal;
use serde::Serialize;
use serde_json::Value;

use super::usages::{Place, place_of};
use super::{AppliedLimit, SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::RenameArgs;
use crate::consts::{ARG_NEW_NAME, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Sites {
    /// All places that have to change, before the limit cut the list.
    total: usize,
    verified: usize,
    unchecked: usize,
    files: usize,
    layers: Vec<LayerRow>,
    items: Vec<SiteRow>,
}

#[derive(Serialize)]
struct Texts {
    /// Whether the project folder was searched: the caller needs `read_source` and the project
    /// must be a folder on this machine.
    searched: bool,
    /// All lines with the old name that no place above explains.
    total: usize,
    /// Files the search could not read.
    skipped: usize,
    items: Vec<TextRow>,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    symbol: Place,
    old_name: String,
    new_name: String,
    /// Whether the symbol is public: code outside the project may use it, and the answer cannot
    /// see that code.
    public_interface: bool,
    /// `true` while a listed place is unchecked by level 2.
    preliminary: bool,
    sites: Sites,
    conflicts: Vec<ConflictRow>,
    text_hits: Texts,
    notice: Option<CodeReport>,
}

fn answer(scope_info: SnapshotInfo, found: &RenamePreview, limit: &AppliedLimit) -> Answer {
    let rows = PreviewRows::of(found);
    let max = usize::try_from(limit.value).unwrap_or(usize::MAX);
    let cut = limit.clamped || found.sites_total > max || found.text.total > max;
    Answer {
        snapshot: scope_info,
        symbol: place_of(&found.symbol.symbol),
        old_name: found.symbol.symbol.name.as_str().to_owned(),
        new_name: found.new_name.clone(),
        public_interface: found.public_interface,
        preliminary: found.verified < found.sites_total,
        sites: Sites {
            total: found.sites_total,
            verified: found.verified,
            unchecked: found.sites_total - found.verified,
            files: found.files,
            layers: rows.layers,
            items: rows.sites,
        },
        conflicts: rows.conflicts,
        text_hits: Texts {
            searched: found.text.searched,
            total: found.text.total,
            skipped: found.text.skipped,
            items: rows.texts,
        },
        notice: cut.then(|| truncation_notice(limit.value, None)),
    }
}

impl Tools {
    /// Shows what renaming a symbol to a new name would hit, and changes nothing: the places that
    /// use the symbol (verified ones apart from unchecked ones), the symbols that already have
    /// the new name, and the lines of the project folder that hold the old name as text (strings,
    /// configuration, documentation), which no index can attribute. Only direct usages count;
    /// code outside the project is not seen, which `public_interface` says. It never says that
    /// the code compiles afterwards: that is the compiler's answer.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid symbol, a new name that
    /// is not a name or is the old one, `MCP-202` if the symbol does not exist, `MCP-201` and
    /// `MCP-203` for the project, `SRC-201` if the project folder cannot be searched, or the
    /// code of a failing port.
    pub fn preview_rename(
        &self,
        principal: &Principal,
        args: &RenameArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let scope = self.scope(args.project)?;
        let found = self.find_symbol(&scope, &args.symbol)?;
        let new_name = args.new_name.trim();
        if !rename::is_valid_name(new_name) || new_name == found.symbol.name.as_str() {
            return Err(McpError::ArgumentsInvalid(ARG_NEW_NAME.to_owned()));
        }
        let limit = self.apply_limit(args.limit);
        let root = rename::searchable_root(*principal, &scope.project.source);
        let preview = rename::preview(
            &RenamePorts {
                index: self.index(),
                usages: self.usage_index(),
                mentions: self.mentions(),
                source: self.source_text(),
            },
            &RenameAsk {
                project: &scope.project,
                snapshot: scope.snapshot.id,
                symbol: &found,
                new_name,
                text_root: root.as_ref(),
                limit: usize::try_from(limit.value).unwrap_or(usize::MAX),
            },
        )?;
        to_json(&answer(scope.info, &preview, &limit))
    }
}
