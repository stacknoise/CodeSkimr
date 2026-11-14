//! The command of the usages screen: who uses a symbol, what it uses and what a change reaches
//! (blueprint 9, mock 6). Every reference says whether it is verified; until level 2 exists
//! (ADR 0033, proposed) none is, and the screen shows that.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, ProjectRepository, SourceText, SymbolKey, SymbolLabel, UsageEntry, UsageIndex,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Confidence, Direction, Permission, Project, ProjectId, SnapshotId, SymbolId, UsageQuery,
    Visibility,
};
use serde::{Deserialize, Serialize};

use super::consts::{
    LEVEL2_UNAVAILABLE, LINE_OFFSET, USAGES_DESCRIPTION, USAGES_LIMIT, USAGES_QUERY, USAGES_TITLE,
};

/// Which way the screen walks the references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Way {
    /// Callers and callees.
    Both,
    /// Only the symbols that use the symbol.
    Incoming,
    /// Only the symbols the symbol uses.
    Outgoing,
}

/// The input of `usages.query`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct UsagesInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the symbol.
    pub symbol: u64,
    /// The directions to walk.
    pub way: Way,
    /// Steps to walk; zero means as deep as the configuration allows.
    pub depth: u32,
    /// Whether references from tests count.
    pub include_tests: bool,
}

/// A symbol on the screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Place {
    /// Id of the symbol.
    pub id: u64,
    /// Qualified name.
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
    /// Name of the layer that owns it.
    pub layer: Option<String>,
}

/// One place where the symbol is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hit {
    /// Step of the walk, from 1.
    pub depth: u32,
    /// Kind label of the use: call, read, test and so on.
    pub kind: &'static str,
    /// `certain` or `probable`.
    pub confidence: &'static str,
    /// Whether level 2 produced the reference.
    pub verified: bool,
    /// File of the occurrence.
    pub path: String,
    /// Line of the occurrence, from 1.
    pub line: u32,
    /// Symbol that contains the occurrence.
    pub from: Place,
    /// Symbol that is used.
    pub to: Place,
}

/// The references of one direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Walk {
    /// Number of references found, also those that were cut.
    pub total: usize,
    /// Verified references first, then by step.
    pub hits: Vec<Hit>,
}

/// How many direct usages there are and how sure they are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Direct usages.
    pub places: usize,
    /// Files that hold them.
    pub files: usize,
    /// Layers that hold them.
    pub layers: usize,
    /// Usages that level 2 verified.
    pub verified: usize,
    /// Usages that only level 1 found.
    pub unchecked: usize,
    /// Of the unchecked ones, those that level 1 itself is unsure about.
    pub probable: usize,
}

/// A layer and the number of symbols a change reaches in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerShare {
    /// Name of the layer.
    pub layer: String,
    /// Number of symbols.
    pub count: usize,
}

/// What a change of the symbol reaches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Impact {
    /// Number of symbols that are reached.
    pub total: usize,
    /// Public symbols among the listed ones: they are part of an interface.
    pub public_interfaces: usize,
    /// Symbols that only verified references lead to.
    pub verified: usize,
    /// Symbols that at least one unchecked reference leads to; the impact is preliminary for them.
    pub unchecked: usize,
    /// The listed symbols per layer.
    pub layers: Vec<LayerShare>,
}

/// The answer of `usages.query`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsagesOut {
    /// Id of the snapshot the answer is from.
    pub snapshot: u64,
    /// The symbol in focus.
    pub symbol: Place,
    /// Signature as written.
    pub signature: Option<String>,
    /// File that declares it.
    pub path: String,
    /// Steps walked.
    pub depth: u32,
    /// Callers, if the direction asked for them.
    pub incoming: Option<Walk>,
    /// Callees, if the direction asked for them.
    pub outgoing: Option<Walk>,
    /// The direct usages counted.
    pub summary: Summary,
    /// What a change reaches.
    pub impact: Impact,
    /// State of level 2 for the project: `unavailable` until it exists.
    pub level2: &'static str,
}

/// What the usages screen reads.
pub struct UsagesPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) usages: Arc<dyn UsageIndex>,
    pub(super) text: Arc<dyn SourceText>,
    pub(super) max_depth: u32,
}

fn place(project: &Project, label: &SymbolLabel) -> Place {
    Place {
        id: label.id.get(),
        qualified_name: label.qualified_name.to_string(),
        kind: label.kind.label(),
        layer: project.layer_name_of(label.qualified_name.as_str()),
    }
}

fn hit(project: &Project, entry: &UsageEntry) -> Hit {
    let reference = &entry.reference;
    Hit {
        depth: entry.depth,
        kind: reference.kind.label(),
        confidence: reference.confidence.label(),
        verified: reference.verified,
        path: entry.path.to_string(),
        line: reference.span.start().line + LINE_OFFSET,
        from: place(project, &entry.from),
        to: place(project, &entry.to),
    }
}

impl UsagesPorts {
    fn walk(
        &self,
        project: &Project,
        snapshot: SnapshotId,
        query: &UsageQuery,
    ) -> Result<Walk, CodedError> {
        let page = self.usages.usages(snapshot, query, USAGES_LIMIT)?;
        let mut hits: Vec<Hit> = page.entries.iter().map(|e| hit(project, e)).collect();
        hits.sort_by_key(|found| (!found.verified, found.depth));
        Ok(Walk {
            total: page.total,
            hits,
        })
    }

    fn summary(incoming: &Walk) -> Summary {
        let direct = || incoming.hits.iter().filter(|found| found.depth == 1);
        let verified = direct().filter(|found| found.verified).count();
        Summary {
            places: direct().count(),
            files: direct()
                .map(|found| &found.path)
                .collect::<BTreeSet<_>>()
                .len(),
            layers: direct()
                .filter_map(|found| found.from.layer.as_ref())
                .collect::<BTreeSet<_>>()
                .len(),
            verified,
            unchecked: direct().count() - verified,
            probable: direct()
                .filter(|found| !found.verified && found.confidence == Confidence::Probable.label())
                .count(),
        }
    }

    fn impact(
        &self,
        project: &Project,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Impact, CodedError> {
        let page = self
            .usages
            .impact(snapshot, symbol, self.max_depth, USAGES_LIMIT)?;
        let mut layers: BTreeMap<String, usize> = BTreeMap::new();
        for reached in &page.symbols {
            if let Some(layer) = project.layer_name_of(reached.label.qualified_name.as_str()) {
                *layers.entry(layer).or_default() += 1;
            }
        }
        let verified = page
            .symbols
            .iter()
            .filter(|reached| reached.verified)
            .count();
        Ok(Impact {
            total: page.total,
            verified,
            unchecked: page.symbols.len() - verified,
            public_interfaces: page
                .symbols
                .iter()
                .filter(|reached| reached.visibility == Some(Visibility::Public))
                .count(),
            layers: layers
                .into_iter()
                .map(|(layer, count)| LayerShare { layer, count })
                .collect(),
        })
    }
}

/// The usages, callees and impact of one symbol.
pub struct UsagesQuery {
    pub(super) ports: Arc<UsagesPorts>,
}

impl UsagesPorts {
    /// Both walks the direction asks for, and the direct usages for the counts.
    pub(super) fn walks(
        &self,
        project: &Project,
        snapshot: SnapshotId,
        input: &UsagesInput,
        depth: u32,
    ) -> Result<(Option<Walk>, Option<Walk>, Walk), CodedError> {
        let query = |direction, depth| UsageQuery {
            symbol: SymbolId::new(input.symbol),
            direction,
            depth,
            include_tests: input.include_tests,
        };
        let wants = |way| input.way == Way::Both || input.way == way;
        let incoming = wants(Way::Incoming)
            .then(|| self.walk(project, snapshot, &query(Direction::Incoming, depth)))
            .transpose()?;
        let outgoing = wants(Way::Outgoing)
            .then(|| self.walk(project, snapshot, &query(Direction::Outgoing, depth)))
            .transpose()?;
        let direct = self.walk(project, snapshot, &query(Direction::Incoming, 1))?;
        Ok((incoming, outgoing, direct))
    }
}

impl Command for UsagesQuery {
    type Input = UsagesInput;
    type Output = UsagesOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(USAGES_QUERY),
        title_key: USAGES_TITLE,
        description_key: USAGES_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, UsagesInput>) -> Result<UsagesOut, CodedError> {
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
        let key = SymbolKey::Id(SymbolId::new(input.symbol));
        let detail = ports
            .index
            .symbol(snapshot, &key)?
            .ok_or_else(|| CodedError::new(ErrorCode::SymbolNotFound))?;
        let depth = if input.depth == 0 {
            ports.max_depth
        } else {
            input.depth.min(ports.max_depth)
        };
        let (incoming, outgoing, direct) = ports.walks(&project, snapshot, &input, depth)?;
        let label = SymbolLabel {
            id: detail.symbol.id,
            qualified_name: detail.symbol.qualified_name.clone(),
            kind: detail.symbol.kind,
        };
        Ok(UsagesOut {
            snapshot: snapshot.get(),
            symbol: place(&project, &label),
            signature: detail.symbol.signature.clone(),
            path: detail.path.to_string(),
            depth,
            incoming,
            outgoing,
            summary: UsagesPorts::summary(&direct),
            impact: ports.impact(&project, snapshot, detail.symbol.id)?,
            level2: LEVEL2_UNAVAILABLE,
        })
    }
}
