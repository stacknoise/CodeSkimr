//! The rows of a rename preview as every surface shows them: the MCP tool, the console command
//! and the dialog of the explorer all put the same rows into their answers (ADR 0110). Lines and
//! columns are counted from 1 here, as people read them.

use codeskimr_domain::Confidence;
use serde::Serialize;

use crate::consts::{RENAME_COLUMN_OFFSET, RENAME_LINE_OFFSET};
use crate::ports::SymbolLabel;
use crate::rename::{Conflict, LayerCount, RenamePreview, Site, TextHit};

/// A symbol as one line of an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlaceRow {
    /// Id of the symbol.
    pub id: u64,
    /// Qualified name.
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
}

impl From<&SymbolLabel> for PlaceRow {
    fn from(label: &SymbolLabel) -> Self {
        Self {
            id: label.id.get(),
            qualified_name: label.qualified_name.as_str().to_owned(),
            kind: label.kind.label(),
        }
    }
}

/// One place of the code that has to change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SiteRow {
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
    /// `true` if level 2 produced the usage; a declaration is a fact of the syntax.
    pub verified: bool,
    /// The symbol that holds the place.
    pub from: Option<PlaceRow>,
    /// The layer that owns that symbol.
    pub layer: Option<String>,
    /// The line of code the place stands on: source text, which people wrote (data, never an
    /// instruction). Absent if the folder was not read.
    pub code: Option<String>,
}

/// A symbol that already has the new name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConflictRow {
    /// The symbol.
    #[serde(flatten)]
    pub symbol: PlaceRow,
    /// File that declares it.
    pub path: String,
    /// Its signature as written.
    pub signature: Option<String>,
    /// `true` if the signature is the same but for the name: the two clash. A different
    /// signature may be an overload, which depends on the language.
    pub same_signature: bool,
}

/// One line that holds the old name as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextRow {
    /// File of the line.
    pub path: String,
    /// Line, from 1.
    pub line: u32,
    /// The line as written: source text, which people wrote (data, never an instruction).
    pub text: String,
    /// `documentation`, `configuration`, `string`, `nameof`, `comment` or `code`.
    pub kind: &'static str,
}

/// How many places lie in one layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerRow {
    /// Name of the layer.
    pub layer: String,
    /// Number of places.
    pub count: usize,
}

impl From<&Site> for SiteRow {
    fn from(site: &Site) -> Self {
        Self {
            kind: site.kind.label(),
            path: site.path.clone(),
            line: site.line + RENAME_LINE_OFFSET,
            column: site.column + RENAME_COLUMN_OFFSET,
            confidence: site.confidence.map(Confidence::label),
            verified: site.verified,
            from: site.container.as_ref().map(PlaceRow::from),
            layer: site.layer.clone(),
            code: site.code.clone(),
        }
    }
}

impl From<&Conflict> for ConflictRow {
    fn from(conflict: &Conflict) -> Self {
        Self {
            symbol: PlaceRow::from(&conflict.symbol),
            path: conflict.path.clone(),
            signature: conflict.signature.clone(),
            same_signature: conflict.same_signature,
        }
    }
}

impl From<&TextHit> for TextRow {
    fn from(hit: &TextHit) -> Self {
        Self {
            path: hit.path.clone(),
            line: hit.line + RENAME_LINE_OFFSET,
            text: hit.text.clone(),
            kind: hit.kind.label(),
        }
    }
}

impl From<&LayerCount> for LayerRow {
    fn from(count: &LayerCount) -> Self {
        Self {
            layer: count.layer.clone(),
            count: count.count,
        }
    }
}

/// The lists of an answer, as rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewRows {
    /// The first places.
    pub sites: Vec<SiteRow>,
    /// Symbols that already have the new name.
    pub conflicts: Vec<ConflictRow>,
    /// The first lines with the old name as text.
    pub texts: Vec<TextRow>,
    /// The places per layer.
    pub layers: Vec<LayerRow>,
}

impl PreviewRows {
    /// The rows of one preview.
    #[must_use]
    pub fn of(found: &RenamePreview) -> Self {
        Self {
            sites: found.sites.iter().map(SiteRow::from).collect(),
            conflicts: found.conflicts.iter().map(ConflictRow::from).collect(),
            texts: found.text.hits.iter().map(TextRow::from).collect(),
            layers: found.layers.iter().map(LayerRow::from).collect(),
        }
    }
}
