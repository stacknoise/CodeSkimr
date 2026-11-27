//! The places behind a dependency between two modules (ADR 0108): the lines where something in
//! one module refers to something in another. The tab "Modules" of the UI, the MCP tool
//! `get_dependencies` and the command `codeskimr modules` show them, so a ring can be ended by
//! going to the places that have to change.

use codeskimr_domain::{SnapshotId, leading_segments};
use serde::Serialize;

use crate::consts::PLACE_LINE_OFFSET;
use crate::error::CodedError;
use crate::ports::ReferencePlaces;

/// The two modules of a dependency, as `get_dependencies` names them for the given `depth`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Between<'a> {
    /// Name of the depending module.
    pub from: &'a str,
    /// Name of the module it depends on.
    pub to: &'a str,
    /// The depth the modules were merged to; zero keeps every namespace.
    pub depth: usize,
}

/// One place where something in a module refers to something in another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlaceOut {
    /// File of the place, relative to the project.
    pub path: String,
    /// Line of the place, counted from 1.
    pub line: u32,
    /// The referring symbol.
    pub from: String,
    /// The symbol it refers to.
    pub to: String,
}

/// The places of one dependency between modules, cut to a limit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PlacesOut {
    /// How many places the dependency has, also those that were cut.
    pub total: usize,
    /// The places in the order of the files, at most the limit.
    pub places: Vec<PlaceOut>,
}

/// The places where a symbol of `between.from` refers to a symbol of `between.to` in a snapshot,
/// the first `limit` of them. They are the references that the count of the dependency adds up:
/// certain ones, without tests and without names that match several symbols.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn places_between(
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
    between: &Between<'_>,
    limit: usize,
) -> Result<PlacesOut, CodedError> {
    let graph = references.namespaces(snapshot)?;
    let owners = graph.owners();
    let module_of = |symbol: &str| {
        let node = graph.namespaces.get(owners.owner_of(symbol)?)?;
        Some(leading_segments(node.name.as_str(), between.depth))
    };
    let mut found = PlacesOut::default();
    references.for_each_reference(snapshot, &mut |place| {
        if module_of(place.from.as_str()) != Some(between.from)
            || module_of(place.to.as_str()) != Some(between.to)
        {
            return;
        }
        found.total += 1;
        if found.places.len() < limit {
            found.places.push(PlaceOut {
                path: place.path.as_str().to_owned(),
                line: place.span.start().line + PLACE_LINE_OFFSET,
                from: place.from.as_str().to_owned(),
                to: place.to.as_str().to_owned(),
            });
        }
    })?;
    Ok(found)
}
