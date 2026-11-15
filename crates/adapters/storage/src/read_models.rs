//! Rebuilding the read models of a snapshot (blueprint 6.2).
//!
//! The scan writes the model, queries read these precomputed tables (light CQRS). They are rebuilt
//! inside the transaction of the snapshot, so a snapshot never exists without them.

use std::collections::BTreeMap;

use codeskimr_domain::{Confidence, Layer, MetricKind, ReferenceKind, SnapshotId, SymbolKind};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from};
use crate::consts::{EDGE_STEP, TYPE_KINDS};
use crate::cycles::edges_in_cycles;
use crate::error::StorageResult;
use crate::references::id_set;
use crate::sql;

/// Fills `search_index`, `usage_counts`, `type_summary` and `layer_graph` of a snapshot.
pub fn rebuild(conn: &Connection, snapshot: SnapshotId, layers: &[Layer]) -> StorageResult<()> {
    let id = column_int(snapshot.get())?;
    conn.execute(
        sql::SEARCH_REBUILD,
        params![id, SymbolKind::Parameter.label()],
    )?;
    rebuild_usages(conn, id)?;
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

/// Fills `usage_counts`: how many references of the snapshot point at each symbol. References of
/// kind `test` are left out, like in coupling; the usages list still shows them. A reference
/// counts when its file and its target are in the snapshot, as in the view `snapshot_references`.
fn rebuild_usages(conn: &Connection, snapshot: i64) -> StorageResult<()> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut counts: BTreeMap<i64, i64> = BTreeMap::new();
    let mut statement = conn.prepare_cached(sql::REFERENCE_TARGETS)?;
    let mut rows = statement.query(params![snapshot, ReferenceKind::Test.label()])?;
    while let Some(row) = rows.next()? {
        let (file, target): (i64, i64) = (row.get(0)?, row.get(1)?);
        if files.contains(&file) && symbols.contains(&target) {
            *counts.entry(target).or_default() += 1;
        }
    }
    let mut insert = conn.prepare_cached(sql::USAGE_INSERT)?;
    for (symbol, usages) in counts {
        insert.execute(params![snapshot, symbol, usages])?;
    }
    Ok(())
}

/// Index of the layer a qualified name belongs to: the first layer with a namespace that equals
/// the name or is a prefix of it up to a separator.
fn layer_of(layers: &[Layer], name: &str) -> Option<usize> {
    layers.iter().position(|layer| layer.owns(name))
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

/// Counts the references between different layers, without those of kind `test` and without
/// those that are only probable: a name that matches several symbols says nothing about the
/// dependency (issue #464).
fn count_edges(
    conn: &Connection,
    snapshot: i64,
    owners: &BTreeMap<u64, usize>,
) -> StorageResult<BTreeMap<(usize, usize), u32>> {
    let mut statement = conn.prepare_cached(sql::REFERENCE_ENDS)?;
    let kinds = params![
        snapshot,
        ReferenceKind::Test.label(),
        Confidence::Certain.label()
    ];
    let rows = statement.query_map(kinds, |row| {
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
            let count: &mut u32 = counts.entry((from, to)).or_default();
            *count += EDGE_STEP;
        }
    }
    Ok(counts)
}

/// Aggregates the references between layers. An edge violates the layering when the layer it
/// starts in limits its dependencies (`may_use`) and does not list the layer it ends in.
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
                    !from_layer.allows(&to_layer.name),
                    cyclic.contains(&(from, to)),
                ],
            )?;
        }
    }
    Ok(())
}
