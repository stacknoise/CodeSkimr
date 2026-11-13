//! Rebuilding the read models of a snapshot (blueprint 6.2).
//!
//! The scan writes the model, queries read these precomputed tables (light CQRS). They are rebuilt
//! inside the transaction of the snapshot, so a snapshot never exists without them.

use std::collections::BTreeMap;

use codeskimr_domain::{Layer, MetricKind, SnapshotId, SymbolKind};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from};
use crate::consts::sql;
use crate::consts::{NAME_SEPARATOR, NOT_VIOLATING, TYPE_KINDS};
use crate::cycles::edges_in_cycles;
use crate::error::StorageResult;

/// Fills `search_index`, `usage_counts`, `type_summary` and `layer_graph` of a snapshot.
pub fn rebuild(conn: &Connection, snapshot: SnapshotId, layers: &[Layer]) -> StorageResult<()> {
    let id = column_int(snapshot.get())?;
    conn.execute(
        sql::SEARCH_REBUILD,
        params![id, SymbolKind::Parameter.label()],
    )?;
    conn.execute(sql::USAGES_REBUILD, params![id])?;
    for kind in TYPE_KINDS.iter().copied() {
        conn.execute(
            sql::TYPE_SUMMARY_REBUILD,
            params![
                id,
                kind.label(),
                MetricKind::Loc.label(),
                MetricKind::CyclomaticComplexity.label(),
                MetricKind::MaintainabilityIndex.label(),
            ],
        )?;
    }
    rebuild_layer_graph(conn, id, layers)
}

/// Index of the layer a qualified name belongs to: the first layer with a namespace that equals
/// the name or is a prefix of it up to a separator.
fn layer_of(layers: &[Layer], name: &str) -> Option<usize> {
    layers.iter().position(|layer| {
        layer.namespaces.iter().any(|namespace| {
            let prefix = namespace.as_str();
            name == prefix
                || name
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with(NAME_SEPARATOR))
        })
    })
}

fn layer_of_symbols(
    conn: &Connection,
    snapshot: i64,
    layers: &[Layer],
) -> StorageResult<BTreeMap<u64, usize>> {
    let mut statement = conn.prepare_cached(sql::SYMBOL_NAMES)?;
    let rows = statement.query_map(params![snapshot], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<String>()?))
    })?;
    let mut owners = BTreeMap::new();
    for row in rows {
        let (id, name) = row?;
        if let Some(layer) = layer_of(layers, &name) {
            owners.insert(id_from(id)?, layer);
        }
    }
    Ok(owners)
}

/// Counts the references between different layers.
fn count_edges(
    conn: &Connection,
    snapshot: i64,
    owners: &BTreeMap<u64, usize>,
) -> StorageResult<BTreeMap<(usize, usize), u32>> {
    let mut statement = conn.prepare_cached(sql::REFERENCE_ENDS)?;
    let rows = statement.query_map(params![snapshot], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<i64>()?))
    })?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (from, to) = row?;
        let ends = (owners.get(&id_from(from)?), owners.get(&id_from(to)?));
        if let (Some(&from), Some(&to)) = ends
            && from != to
        {
            *counts.entry((from, to)).or_insert(0_u32) += 1;
        }
    }
    Ok(counts)
}

/// Aggregates the references between layers. `violates_layering` stays false: the allowed
/// directions are not part of the project model yet.
fn rebuild_layer_graph(conn: &Connection, snapshot: i64, layers: &[Layer]) -> StorageResult<()> {
    if layers.is_empty() {
        return Ok(());
    }
    let owners = layer_of_symbols(conn, snapshot, layers)?;
    let counts = count_edges(conn, snapshot, &owners)?;
    let pairs: Vec<(usize, usize)> = counts.keys().copied().collect();
    let cyclic = edges_in_cycles(&pairs);
    for (&(from, to), &count) in &counts {
        let names = (layers.get(from), layers.get(to));
        if let (Some(from_layer), Some(to_layer)) = names {
            conn.execute(
                sql::EDGE_INSERT,
                params![
                    snapshot,
                    from_layer.name.as_str(),
                    to_layer.name.as_str(),
                    i64::from(count),
                    NOT_VIOLATING,
                    cyclic.contains(&(from, to)),
                ],
            )?;
        }
    }
    Ok(())
}
