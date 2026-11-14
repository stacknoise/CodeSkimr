//! The level "types" of the dependency map: one type in the middle, the types that use it in the
//! row above, the types it uses in the row below. Placed by the same layout as the other levels,
//! with the rows fixed (blueprint 15.5, screen 3, "type focus").

use std::collections::{BTreeMap, BTreeSet};

use codeskimr_application::config::Graph;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{TypeFocus, TypeNeighbour};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    LayoutEdge, LayoutGraph, LayoutNode, NodeIndex, Project, SnapshotId, SymbolId,
};

use super::consts::{
    BOX_ID_SEPARATOR, TYPE_ROW_FOCUS, TYPE_ROW_USED, TYPE_ROW_USERS, TYPE_SIDE_MAX,
};
use super::graph::{BoxOut, FocusOut, GraphOut, GraphPorts, LinkOut};

/// One type that is drawn: its id, qualified name, the name of its box and its row. The box name
/// is the qualified name, with the id added if another drawn type has the same one, so that the
/// links and the selection of the picture can tell the two apart.
struct Drawn {
    symbol: u64,
    qualified: String,
    name: String,
    row: u32,
}

/// The types that are drawn: the focus, then the users, then the used types that are not users
/// as well (a type on both sides stays in the row of the users and gets two links).
fn drawn(found: &TypeFocus) -> Vec<Drawn> {
    let mut seen = BTreeSet::new();
    let mut all = vec![Drawn {
        symbol: found.focus.id.get(),
        qualified: found.focus.qualified_name.to_string(),
        name: String::new(),
        row: TYPE_ROW_FOCUS,
    }];
    seen.insert(found.focus.id.get());
    for (side, row) in [(&found.users, TYPE_ROW_USERS), (&found.used, TYPE_ROW_USED)] {
        for neighbour in side.iter().take(TYPE_SIDE_MAX) {
            if seen.insert(neighbour.label.id.get()) {
                all.push(Drawn {
                    symbol: neighbour.label.id.get(),
                    qualified: neighbour.label.qualified_name.to_string(),
                    name: String::new(),
                    row,
                });
            }
        }
    }
    name_boxes(&mut all);
    all
}

/// Gives every drawn type the name of its box.
fn name_boxes(all: &mut [Drawn]) {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for one in all.iter() {
        *count.entry(one.qualified.clone()).or_default() += 1;
    }
    for one in all.iter_mut() {
        one.name = if count.get(&one.qualified).copied().unwrap_or(0) > 1 {
            format!("{}{BOX_ID_SEPARATOR}{}", one.qualified, one.symbol)
        } else {
            one.qualified.clone()
        };
    }
}

fn links(found: &TypeFocus, types: &[Drawn]) -> Vec<LinkOut> {
    let boxed = |id: SymbolId| {
        types
            .iter()
            .find(|one| one.symbol == id.get())
            .map(|one| one.name.as_str())
    };
    let name = |label: &str, other: &str, count, towards_focus: bool| {
        let (from, to) = if towards_focus {
            (label, other)
        } else {
            (other, label)
        };
        LinkOut {
            from: from.to_owned(),
            to: to.to_owned(),
            count,
            violates_layering: false,
            in_cycle: false,
        }
    };
    let middle = boxed(found.focus.id).unwrap_or_default();
    let side = |list: &[TypeNeighbour], towards_focus: bool| -> Vec<LinkOut> {
        list.iter()
            .take(TYPE_SIDE_MAX)
            .filter_map(|one| {
                let label = boxed(one.label.id)?;
                Some(name(label, middle, one.count, towards_focus))
            })
            .collect()
    };
    let mut all = side(&found.users, true);
    all.extend(side(&found.used, false));
    all
}

fn place(look: &Graph, types: &[Drawn], links: &[LinkOut]) -> Result<Vec<BoxOut>, CodedError> {
    let size = look.node_size();
    let nodes = types
        .iter()
        .map(|one| LayoutNode {
            size,
            rank: Some(one.row),
        })
        .collect();
    let position = |name: &str| types.iter().position(|one| one.name == name);
    let edges = links
        .iter()
        .filter_map(|link| {
            Some(LayoutEdge {
                from: NodeIndex::new(position(&link.from)?),
                to: NodeIndex::new(position(&link.to)?),
            })
        })
        .collect();
    let layout = LayoutGraph::new(nodes, edges)
        .map_err(CodedError::caused_by)?
        .layout(&look.layout_params());
    Ok(types
        .iter()
        .zip(&layout.placements)
        .map(|(one, spot)| BoxOut {
            name: one.name.clone(),
            x: spot.x,
            y: spot.y,
            width: size.width,
            height: size.height,
            rank: spot.rank,
            order: spot.order,
            layer: None,
            types: None,
            symbol: Some(one.symbol),
        })
        .collect())
}

/// The map around one type. A symbol that is not a type is `MCP-202`, like an unknown one.
pub(super) fn types_out(
    ports: &GraphPorts,
    project: &Project,
    snapshot: SnapshotId,
    focus: Option<u64>,
) -> Result<GraphOut, CodedError> {
    let found = focus
        .and_then(|id| {
            ports
                .index
                .type_focus(snapshot, SymbolId::new(id))
                .transpose()
        })
        .transpose()?
        .ok_or_else(|| CodedError::new(ErrorCode::SymbolNotFound))?;
    let types = drawn(&found);
    let links = links(&found, &types);
    let mut boxes = place(&ports.look, &types, &links)?;
    for (one, drawn) in boxes.iter_mut().zip(&types) {
        one.layer = project.layer_name_of(&drawn.qualified);
    }
    let reach = |pick: fn(&BoxOut) -> u32| boxes.iter().map(pick).max().unwrap_or(0);
    let hidden = |list: &[TypeNeighbour]| u32::try_from(list.len().saturating_sub(TYPE_SIDE_MAX));
    Ok(GraphOut {
        width: reach(|one| one.x + one.width),
        height: reach(|one| one.y + one.height),
        focus: Some(FocusOut {
            symbol: found.focus.id.get(),
            name: types
                .first()
                .map(|one| one.name.clone())
                .unwrap_or_default(),
            users_more: hidden(&found.users).unwrap_or(u32::MAX),
            used_more: hidden(&found.used).unwrap_or(u32::MAX),
        }),
        boxes,
        links,
        ..GraphOut::default()
    })
}
