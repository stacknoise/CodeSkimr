//! Rebuilding the read models of a snapshot (blueprint 6.2).
//!
//! The scan writes the model, queries read these precomputed tables (light CQRS). They are rebuilt
//! inside the transaction of the snapshot, so a snapshot never exists without them.

use std::collections::BTreeMap;

use codeskimr_domain::{
    Confidence, Layer, MetricKind, ProjectId, ReferenceKind, SnapshotId, SymbolKind,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from};
use crate::consts::{EDGE_STEP, REFERENCE_KIND_COLUMN, TYPE_KINDS};
use crate::cycles::edges_in_cycles;
use crate::error::StorageResult;
use crate::previous::latest_snapshot_id;
use crate::references::{id_set, of_snapshot};
use crate::sql;

/// Fills `search_index`, `usage_counts`, `type_summary` and `layer_graph` of a snapshot. Returns
/// how many references of kind `test` the snapshot holds, which the usage counts leave out.
pub fn rebuild(conn: &Connection, snapshot: SnapshotId, layers: &[Layer]) -> StorageResult<u64> {
    let id = column_int(snapshot.get())?;
    conn.execute(
        sql::SEARCH_REBUILD,
        params![id, SymbolKind::Parameter.label()],
    )?;
    let tests = rebuild_usages(conn, id)?;
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
    rebuild_layer_graph(conn, id, layers)?;
    Ok(tests)
}

/// Fills `usage_counts`: how many references of the snapshot point at each symbol. References of
/// kind `test` are left out, like in coupling; the usages list still shows them. A reference
/// counts when its file and its target are in the snapshot, as in the view `snapshot_references`.
/// Returns the number of references of kind `test` that count in this sense.
fn rebuild_usages(conn: &Connection, snapshot: i64) -> StorageResult<u64> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut counts: BTreeMap<i64, i64> = BTreeMap::new();
    let mut statement =
        conn.prepare_cached(&of_snapshot(conn, snapshot, sql::REFERENCE_TARGETS)?)?;
    let mut tests = u64::default();
    let mut rows = statement.query(params![snapshot])?;
    while let Some(row) = rows.next()? {
        let (file, target): (i64, i64) = (row.get(0)?, row.get(1)?);
        if !(files.contains(&file) && symbols.contains(&target)) {
            continue;
        }
        let kind = row
            .get_ref(REFERENCE_KIND_COLUMN)?
            .as_str()
            .map_err(rusqlite::Error::from)?;
        if kind == ReferenceKind::Test.label() {
            tests += 1;
        } else {
            *counts.entry(target).or_default() += 1;
        }
    }
    let mut insert = conn.prepare_cached(sql::USAGE_INSERT)?;
    for (symbol, usages) in counts {
        insert.execute(params![snapshot, symbol, usages])?;
    }
    Ok(tests)
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
    let bind = params![
        snapshot,
        ReferenceKind::Test.label(),
        Confidence::Certain.label()
    ];
    let rows = statement.query_map(bind, |row| {
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

/// Builds the layer graph of the newest snapshot of `project` again, for `layers` that changed
/// since it was written. Nothing happens if the project has no snapshot.
pub fn relayer(conn: &Connection, project: ProjectId, layers: &[Layer]) -> StorageResult<()> {
    let Some(snapshot) = latest_snapshot_id(conn, project)? else {
        return Ok(());
    };
    conn.execute(sql::LAYER_GRAPH_DELETE, params![snapshot])?;
    rebuild_layer_graph(conn, snapshot, layers)
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
