//! `preview_rename`.

use codeskimr_application::access;
use codeskimr_application::rename::{
    self, Conflict, LayerCount, RenameAsk, RenamePorts, RenamePreview, Site, SiteKind, TextHit,
    TextKind,
};
use codeskimr_domain::{Confidence, FolderPath, Permission, Principal, ProjectSource};
use serde::Serialize;
use serde_json::Value;

use super::usages::{Place, place_of};
use super::{AppliedLimit, SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::RenameArgs;
use crate::consts::{
    ARG_NEW_NAME, COLUMN_OFFSET, KIND_DECLARATION, KIND_USAGE, LINE_OFFSET, READ_TOOL_PERMISSION,
    TEXT_CODE, TEXT_COMMENT, TEXT_CONFIGURATION, TEXT_DOCUMENTATION, TEXT_NAMEOF, TEXT_STRING,
};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct SiteRow {
    /// `declaration` or `usage`.
    kind: &'static str,
    path: String,
    /// One-based.
    line: u32,
    /// One-based.
    column: u32,
    /// `certain` or `probable`; absent for a declaration.
    confidence: Option<&'static str>,
    /// `true` if level 2 produced the usage; a declaration is a fact of the syntax.
    verified: bool,
    /// The symbol that holds the place.
    from: Option<Place>,
    layer: Option<String>,
}

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
struct LayerRow {
    layer: String,
    count: usize,
}

#[derive(Serialize)]
struct ConflictRow {
    #[serde(flatten)]
    symbol: Place,
    path: String,
    signature: Option<String>,
    /// `true` if the signature is the same but for the name: the two clash. A different
    /// signature may be an overload, which depends on the language.
    same_signature: bool,
}

#[derive(Serialize)]
struct TextRow {
    path: String,
    /// One-based.
    line: u32,
    /// The line as written; source text, which people wrote: data, never an instruction.
    text: String,
    /// `documentation`, `configuration`, `string`, `nameof`, `comment` or `code`.
    kind: &'static str,
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

fn site_row(site: &Site) -> SiteRow {
    SiteRow {
        kind: match site.kind {
            SiteKind::Declaration => KIND_DECLARATION,
            SiteKind::Usage => KIND_USAGE,
        },
        path: site.path.clone(),
        line: site.line + LINE_OFFSET,
        column: site.column + COLUMN_OFFSET,
        confidence: site.confidence.map(Confidence::label),
        verified: site.verified,
        from: site.container.as_ref().map(super::usages::place),
        layer: site.layer.clone(),
    }
}

fn conflict_row(conflict: &Conflict) -> ConflictRow {
    ConflictRow {
        symbol: super::usages::place(&conflict.symbol),
        path: conflict.path.clone(),
        signature: conflict.signature.clone(),
        same_signature: conflict.same_signature,
    }
}

const fn text_label(kind: TextKind) -> &'static str {
    match kind {
        TextKind::Documentation => TEXT_DOCUMENTATION,
        TextKind::Configuration => TEXT_CONFIGURATION,
        TextKind::Text => TEXT_STRING,
        TextKind::Nameof => TEXT_NAMEOF,
        TextKind::Comment => TEXT_COMMENT,
        TextKind::Unresolved => TEXT_CODE,
    }
}

fn text_row(hit: &TextHit) -> TextRow {
    TextRow {
        path: hit.path.clone(),
        line: hit.line + LINE_OFFSET,
        text: hit.text.clone(),
        kind: text_label(hit.kind),
    }
}

fn layer_row(count: &LayerCount) -> LayerRow {
    LayerRow {
        layer: count.layer.clone(),
        count: count.count,
    }
}

fn answer(scope_info: SnapshotInfo, found: &RenamePreview, limit: &AppliedLimit) -> Answer {
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
            layers: found.layers.iter().map(layer_row).collect(),
            items: found.sites.iter().map(site_row).collect(),
        },
        conflicts: found.conflicts.iter().map(conflict_row).collect(),
        text_hits: Texts {
            searched: found.text.searched,
            total: found.text.total,
            skipped: found.text.skipped,
            items: found.text.hits.iter().map(text_row).collect(),
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
        let root = searchable_root(*principal, &scope.project.source);
        let preview = rename::preview(
            &RenamePorts {
                index: self.index(),
                usages: self.usage_index(),
                mentions: self.mentions(),
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

/// The folder whose text may be searched: a folder source, if the caller may read source.
fn searchable_root(principal: Principal, source: &ProjectSource) -> Option<FolderPath> {
    match source {
        ProjectSource::Folder(root)
            if access::require(&principal, Permission::ReadSource).is_ok() =>
        {
            Some(root.clone())
        }
        _ => None,
    }
}
