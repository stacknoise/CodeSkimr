//! SQL statements. Parameters are numbered; every statement documents what they are.

/// A registered project by id: id.
pub const PROJECT_BY_ID: &str =
    "SELECT name, source_kind, location, branch, credential FROM projects WHERE id = ?1";
/// Id of the project with the given source: kind, location, branch.
pub const PROJECT_BY_SOURCE: &str =
    "SELECT id FROM projects WHERE source_kind = ?1 AND location = ?2 AND branch = ?3";
/// Inserts or updates a project: id, name, kind, location, branch, credential name.
pub const PROJECT_UPSERT: &str = "INSERT INTO projects (id, name, source_kind, location, branch, credential) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
    ON CONFLICT (id) DO UPDATE SET name = excluded.name, source_kind = excluded.source_kind, \
    location = excluded.location, branch = excluded.branch, credential = excluded.credential";
/// Whether a project exists: id.
pub const PROJECT_EXISTS: &str = "SELECT COUNT(*) FROM projects WHERE id = ?1";
/// Removes the layers of a project (their namespaces go with them): project id.
pub const LAYERS_DELETE: &str = "DELETE FROM layers WHERE project_id = ?1";
/// Adds a layer: project id, position, name.
pub const LAYER_INSERT: &str =
    "INSERT INTO layers (project_id, position, name) VALUES (?1, ?2, ?3)";
/// Adds a namespace to a layer: project id, layer position, position, namespace.
pub const NAMESPACE_INSERT: &str = "INSERT INTO layer_namespaces \
    (project_id, layer_position, position, namespace) VALUES (?1, ?2, ?3, ?4)";
/// Layers of a project in order: project id.
pub const LAYERS_OF_PROJECT: &str =
    "SELECT position, name FROM layers WHERE project_id = ?1 ORDER BY position";
/// Namespaces of one layer in order: project id, layer position.
pub const NAMESPACES_OF_LAYER: &str = "SELECT namespace FROM layer_namespaces \
    WHERE project_id = ?1 AND layer_position = ?2 ORDER BY position";

/// Highest id of any stored snapshot, file, symbol or reference.
pub const MAX_ID: &str = "SELECT MAX(id) FROM (\
    SELECT MAX(id) AS id FROM snapshots UNION ALL SELECT MAX(id) FROM files \
    UNION ALL SELECT MAX(id) FROM symbols UNION ALL SELECT MAX(id) FROM symbol_references)";

/// Highest id of any registered project.
pub const MAX_PROJECT_ID: &str = "SELECT MAX(id) FROM projects";

/// Files of the newest snapshot of a project: project id.
pub const LATEST_FILES: &str = "SELECT snapshot_id, id, path, language, content_hash, loc \
    FROM files WHERE snapshot_id = (SELECT MAX(id) FROM snapshots WHERE project_id = ?1) \
    ORDER BY path";
/// Id of the newest snapshot of a project, null without one: project id.
pub const LATEST_SNAPSHOT_ID: &str = "SELECT MAX(id) FROM snapshots WHERE project_id = ?1";
/// The newest snapshot of a project: project id.
pub const LATEST_SNAPSHOT: &str = "SELECT id, commit_hash, created, mode, files, \
    symbols, refs FROM snapshots WHERE project_id = ?1 ORDER BY id DESC LIMIT 1";

/// Adds a snapshot header: id, project, commit, created, mode, files, symbols, references.
pub const SNAPSHOT_INSERT: &str = "INSERT INTO snapshots \
    (id, project_id, commit_hash, created, mode, files, symbols, refs) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
/// Writes the final totals: id, files, symbols, references.
pub const SNAPSHOT_TOTALS: &str =
    "UPDATE snapshots SET files = ?2, symbols = ?3, refs = ?4 WHERE id = ?1";
/// Newest snapshot of a project other than the given one: project id, snapshot id.
pub const PREVIOUS_SNAPSHOT: &str =
    "SELECT MAX(id) FROM snapshots WHERE project_id = ?1 AND id <> ?2";
/// Deletes all but the newest snapshots of a project: project id, number to keep.
pub const SNAPSHOT_PRUNE: &str = "DELETE FROM snapshots WHERE id = ?1";
/// The snapshots of a project that are older than the newest ones: project id, number to keep.
pub const SNAPSHOTS_EXPIRED: &str = "SELECT id FROM snapshots WHERE project_id = ?1 AND id NOT IN \
    (SELECT id FROM snapshots WHERE project_id = ?1 ORDER BY id DESC LIMIT ?2)";
/// What a snapshot owns, children before parents: each takes the snapshot id. Deleting them one
/// table at a time is much faster than the cascade of the foreign keys, which looks up the
/// children of every single row (issue #108).
pub const SNAPSHOT_CONTENT_DELETE: &[&str] = &[
    "DELETE FROM fingerprints WHERE snapshot_id = ?1",
    "DELETE FROM metrics WHERE snapshot_id = ?1",
    "DELETE FROM symbol_references WHERE snapshot_id = ?1",
    "DELETE FROM type_summary WHERE snapshot_id = ?1",
    "DELETE FROM search_index WHERE snapshot_id = ?1",
    "DELETE FROM usage_counts WHERE snapshot_id = ?1",
    "DELETE FROM layer_graph WHERE snapshot_id = ?1",
    "DELETE FROM symbols WHERE snapshot_id = ?1",
    "DELETE FROM files WHERE snapshot_id = ?1",
];

/// Adds a file: snapshot, id, path, language, hash, loc.
pub const FILE_INSERT: &str = "INSERT INTO files \
    (snapshot_id, id, path, language, content_hash, loc) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
/// Adds a symbol: snapshot, id, file, kind, name, qualified name, parent, visibility, static,
/// start line and column, end line and column, signature, doc.
pub const SYMBOL_INSERT: &str = "INSERT INTO symbols (snapshot_id, id, file_id, kind, name, \
    qualified_name, parent_id, visibility, is_static, start_line, start_column, end_line, \
    end_column, signature, doc) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)";
/// Adds a reference: snapshot, id, from, to, file, start line and column, end line and column,
/// kind, confidence.
pub const REFERENCE_INSERT: &str = "INSERT INTO symbol_references (snapshot_id, id, from_symbol, \
    to_symbol, file_id, start_line, start_column, end_line, end_column, kind, confidence) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
/// Adds a metric: snapshot, symbol, kind, value.
pub const METRIC_INSERT: &str =
    "INSERT INTO metrics (snapshot_id, symbol_id, kind, value) VALUES (?1, ?2, ?3, ?4)";

/// Adds a fingerprint: snapshot, symbol, position, hash.
pub const FINGERPRINT_INSERT: &str = "INSERT INTO fingerprints \
    (snapshot_id, symbol_id, position, hash) VALUES (?1, ?2, ?3, ?4)";
/// Copies the fingerprints of the carried symbols: previous, new.
pub const CARRY_FINGERPRINTS: &str = "INSERT INTO fingerprints \
    (snapshot_id, symbol_id, position, hash) \
    SELECT ?2, f.symbol_id, f.position, f.hash FROM fingerprints f WHERE f.snapshot_id = ?1 \
    AND f.symbol_id IN (SELECT id FROM symbols WHERE snapshot_id = ?2)";
/// The fingerprints of one symbol in order: snapshot, symbol.
pub const FINGERPRINTS_OF: &str = "SELECT position, hash FROM fingerprints \
    WHERE snapshot_id = ?1 AND symbol_id = ?2 ORDER BY position";
/// Per symbol, how many distinct hashes of a JSON list it holds: snapshot, list.
pub const FINGERPRINTS_SHARED: &str = "SELECT symbol_id, COUNT(DISTINCT hash) FROM fingerprints \
    WHERE snapshot_id = ?1 AND hash IN (SELECT value FROM json_each(?2)) GROUP BY symbol_id";
/// Per symbol of a JSON list, how many distinct hashes it holds: snapshot, list.
pub const FINGERPRINTS_SIZES: &str = "SELECT symbol_id, COUNT(DISTINCT hash) FROM fingerprints \
    WHERE snapshot_id = ?1 AND symbol_id IN (SELECT value FROM json_each(?2)) \
    GROUP BY symbol_id";
/// The label of the symbols of a JSON list: snapshot, list.
pub const SIMILAR_LABELS: &str = "SELECT s.id, s.qualified_name, s.kind, f.path, s.start_line, \
    s.start_column, s.end_line, s.end_column FROM symbols s \
    JOIN files f ON f.snapshot_id = s.snapshot_id AND f.id = s.file_id \
    WHERE s.snapshot_id = ?1 AND s.id IN (SELECT value FROM json_each(?2))";

/// Removes a carried file that no longer exists: snapshot, path.
pub const FILE_REMOVE: &str = "DELETE FROM files WHERE snapshot_id = ?1 AND path = ?2";
/// Copies the files of the previous snapshot that were not analysed again: previous, new.
pub const CARRY_FILES: &str = "INSERT INTO files \
    (snapshot_id, id, path, language, content_hash, loc) \
    SELECT ?2, id, path, language, content_hash, loc FROM files \
    WHERE snapshot_id = ?1 AND path NOT IN (SELECT path FROM files WHERE snapshot_id = ?2)";
/// Copies the symbols of the carried files: previous, new.
pub const CARRY_SYMBOLS: &str = "INSERT INTO symbols (snapshot_id, id, file_id, kind, name, \
    qualified_name, parent_id, visibility, is_static, start_line, start_column, end_line, \
    end_column, signature, doc) \
    SELECT ?2, s.id, s.file_id, s.kind, s.name, s.qualified_name, s.parent_id, s.visibility, \
    s.is_static, s.start_line, s.start_column, s.end_line, s.end_column, s.signature, s.doc \
    FROM symbols s WHERE s.snapshot_id = ?1 AND s.file_id IN \
    (SELECT id FROM files WHERE snapshot_id = ?2) \
    AND s.file_id IN (SELECT id FROM files WHERE snapshot_id = ?1)";
/// Copies the references whose both ends survived: previous, new.
pub const CARRY_REFERENCES: &str = "INSERT INTO symbol_references (snapshot_id, id, from_symbol, \
    to_symbol, file_id, start_line, start_column, end_line, end_column, kind, confidence) \
    SELECT ?2, r.id, r.from_symbol, r.to_symbol, r.file_id, r.start_line, r.start_column, \
    r.end_line, r.end_column, r.kind, r.confidence FROM symbol_references r \
    WHERE r.snapshot_id = ?1 \
    AND r.file_id IN (SELECT id FROM files WHERE snapshot_id = ?2) \
    AND r.from_symbol IN (SELECT id FROM symbols WHERE snapshot_id = ?2) \
    AND r.to_symbol IN (SELECT id FROM symbols WHERE snapshot_id = ?2)";
/// Copies the metrics of the carried symbols, except those the new snapshot has already:
/// previous, new.
pub const CARRY_METRICS: &str = "INSERT OR IGNORE INTO metrics (snapshot_id, symbol_id, kind, value) \
    SELECT ?2, m.symbol_id, m.kind, m.value FROM metrics m WHERE m.snapshot_id = ?1 \
    AND m.symbol_id IN (SELECT id FROM symbols WHERE snapshot_id = ?2)";

/// Number of files, symbols and references of a snapshot: snapshot.
pub const TOTALS: &str = "SELECT (SELECT COUNT(*) FROM files WHERE snapshot_id = ?1), \
    (SELECT COUNT(*) FROM symbols WHERE snapshot_id = ?1), \
    (SELECT COUNT(*) FROM symbol_references WHERE snapshot_id = ?1)";

/// Fills `search_index`: snapshot, kind that is left out.
pub const SEARCH_REBUILD: &str = "INSERT INTO search_index \
    (snapshot_id, symbol_id, name_lower, name, qualified_name, kind, path) \
    SELECT s.snapshot_id, s.id, lower(s.name), s.name, s.qualified_name, s.kind, f.path \
    FROM symbols s JOIN files f ON f.snapshot_id = s.snapshot_id AND f.id = s.file_id \
    WHERE s.snapshot_id = ?1 AND s.kind <> ?2";
/// Fills `usage_counts`: snapshot.
pub const USAGES_REBUILD: &str = "INSERT INTO usage_counts (snapshot_id, symbol_id, usages) \
    SELECT snapshot_id, to_symbol, COUNT(*) FROM symbol_references \
    WHERE snapshot_id = ?1 GROUP BY to_symbol";
/// Fills `type_summary` for one kind: snapshot, kind, LOC metric, complexity metric,
/// maintainability metric.
pub const TYPE_SUMMARY_REBUILD: &str = "INSERT INTO type_summary \
    (snapshot_id, symbol_id, member_count, loc, max_complexity, maintainability) \
    SELECT t.snapshot_id, t.id, COUNT(m.id), MAX(l.value), MAX(c.value), MAX(i.value) \
    FROM symbols t \
    LEFT JOIN symbols m ON m.snapshot_id = t.snapshot_id AND m.parent_id = t.id \
    LEFT JOIN metrics c ON c.snapshot_id = m.snapshot_id AND c.symbol_id = m.id AND c.kind = ?4 \
    LEFT JOIN metrics l ON l.snapshot_id = t.snapshot_id AND l.symbol_id = t.id AND l.kind = ?3 \
    LEFT JOIN metrics i ON i.snapshot_id = t.snapshot_id AND i.symbol_id = t.id AND i.kind = ?5 \
    WHERE t.snapshot_id = ?1 AND t.kind = ?2 GROUP BY t.id";
/// Names of the symbols of a snapshot: snapshot.
pub const SYMBOL_NAMES: &str = "SELECT id, qualified_name FROM symbols WHERE snapshot_id = ?1";
/// Ends of all references of a snapshot: snapshot.
pub const REFERENCE_ENDS: &str =
    "SELECT from_symbol, to_symbol FROM symbol_references WHERE snapshot_id = ?1";
/// Adds a layer graph edge: snapshot, from, to, count, violates, in cycle.
pub const EDGE_INSERT: &str = "INSERT INTO layer_graph \
    (snapshot_id, from_layer, to_layer, edge_count, violates_layering, in_cycle) \
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

/// Usages of a symbol: snapshot, symbol.
pub const USAGES_OF: &str =
    "SELECT usages FROM usage_counts WHERE snapshot_id = ?1 AND symbol_id = ?2";
/// Fuzzy search: snapshot, subsequence pattern, exact text, prefix pattern, substring text,
/// limit. Exact matches come first, then prefix, substring and the rest, shorter names first.
pub const SEARCH: &str = "SELECT i.symbol_id, i.name, i.qualified_name, i.kind, i.path, s.parent_id \
    FROM search_index i JOIN symbols s ON s.snapshot_id = i.snapshot_id AND s.id = i.symbol_id \
    WHERE i.snapshot_id = ?1 AND i.name_lower LIKE ?2 ESCAPE '\\' \
    ORDER BY CASE WHEN i.name_lower = ?3 THEN 0 WHEN i.name_lower LIKE ?4 ESCAPE '\\' THEN 1 \
    WHEN instr(i.name_lower, ?5) > 0 THEN 2 ELSE 3 END, length(i.name_lower), i.name_lower \
    LIMIT ?6";
/// Layer graph of a snapshot: snapshot.
pub const EDGES: &str = "SELECT from_layer, to_layer, edge_count, violates_layering, in_cycle \
    FROM layer_graph WHERE snapshot_id = ?1 ORDER BY from_layer, to_layer";
/// Symbols outside a scope (a symbol and everything it contains) that the scope uses: snapshot,
/// scope symbol, whether tests count, label of the test kind.
pub const DEPENDENCIES_OUTGOING: &str = "WITH RECURSIVE scope(id) AS (SELECT ?2 UNION SELECT s.id FROM symbols s \
    JOIN scope c ON s.parent_id = c.id WHERE s.snapshot_id = ?1) \
    SELECT t.id, t.qualified_name, t.kind, f.path, COUNT(*) FROM symbol_references r \
    JOIN symbols t ON t.snapshot_id = r.snapshot_id AND t.id = r.to_symbol \
    JOIN files f ON f.snapshot_id = t.snapshot_id AND f.id = t.file_id \
    WHERE r.snapshot_id = ?1 AND r.from_symbol IN (SELECT id FROM scope) \
    AND r.to_symbol NOT IN (SELECT id FROM scope) AND (?3 OR r.kind <> ?4) \
    GROUP BY t.id ORDER BY COUNT(*) DESC, t.qualified_name, t.id";
/// Symbols outside a scope (a symbol and everything it contains) that use the scope: snapshot,
/// scope symbol, whether tests count, label of the test kind.
pub const DEPENDENCIES_INCOMING: &str = "WITH RECURSIVE scope(id) AS (SELECT ?2 UNION SELECT s.id FROM symbols s \
    JOIN scope c ON s.parent_id = c.id WHERE s.snapshot_id = ?1) \
    SELECT t.id, t.qualified_name, t.kind, f.path, COUNT(*) FROM symbol_references r \
    JOIN symbols t ON t.snapshot_id = r.snapshot_id AND t.id = r.from_symbol \
    JOIN files f ON f.snapshot_id = t.snapshot_id AND f.id = t.file_id \
    WHERE r.snapshot_id = ?1 AND r.to_symbol IN (SELECT id FROM scope) \
    AND r.from_symbol NOT IN (SELECT id FROM scope) AND (?3 OR r.kind <> ?4) \
    GROUP BY t.id ORDER BY COUNT(*) DESC, t.qualified_name, t.id";
/// Summary of one type: snapshot, symbol.
pub const TYPE_SUMMARY: &str = "SELECT member_count, loc, max_complexity, maintainability \
    FROM type_summary WHERE snapshot_id = ?1 AND symbol_id = ?2";

/// Start of the lookup of one symbol with its file path: snapshot. A condition from
/// `SYMBOL_BY_ID` or `SYMBOL_BY_NAME` follows.
pub const SYMBOL_SELECT: &str = "SELECT s.id, s.file_id, s.kind, s.name, s.qualified_name, \
    s.parent_id, s.visibility, s.is_static, s.start_line, s.start_column, s.end_line, \
    s.end_column, s.signature, s.doc, f.path FROM symbols s \
    JOIN files f ON f.snapshot_id = s.snapshot_id AND f.id = s.file_id \
    WHERE s.snapshot_id = ?1 AND ";
/// Condition of `SYMBOL_SELECT`: symbol id.
pub const SYMBOL_BY_ID: &str = "s.id = ?2";
/// Condition of `SYMBOL_SELECT`: qualified name; the lowest id wins.
pub const SYMBOL_BY_NAME: &str = "s.qualified_name = ?2 ORDER BY s.id LIMIT 1";
/// Condition of `SYMBOL_SELECT`: every symbol of the snapshot, in id order.
pub const SYMBOL_ALL: &str = "1 = 1 ORDER BY s.id";
/// All references of a snapshot: snapshot.
pub const REFERENCES_OF_SNAPSHOT: &str = "SELECT id, from_symbol, to_symbol, file_id, start_line, \
    start_column, end_line, end_column, kind, confidence FROM symbol_references \
    WHERE snapshot_id = ?1 ORDER BY id";
/// Direct members of a symbol: snapshot, symbol.
pub const MEMBERS_OF: &str =
    "SELECT id, name, kind FROM symbols WHERE snapshot_id = ?1 AND parent_id = ?2 ORDER BY id";
/// Metrics of a symbol: snapshot, symbol.
pub const METRICS_OF: &str =
    "SELECT kind, value FROM metrics WHERE snapshot_id = ?1 AND symbol_id = ?2 ORDER BY kind";
/// A symbol as a result line, with its file path: snapshot, symbol.
pub const SYMBOL_LABEL: &str = "SELECT s.qualified_name, s.kind, s.visibility, f.path FROM symbols s \
    JOIN files f ON f.snapshot_id = s.snapshot_id AND f.id = s.file_id \
    WHERE s.snapshot_id = ?1 AND s.id = ?2";
/// Path of a file: snapshot, file.
pub const FILE_PATH: &str = "SELECT path FROM files WHERE snapshot_id = ?1 AND id = ?2";
/// Whether a snapshot is stored: snapshot.
pub const SNAPSHOT_EXISTS: &str = "SELECT COUNT(*) FROM snapshots WHERE id = ?1";

/// Ids of all registered projects in registration order.
pub const PROJECT_IDS: &str = "SELECT id FROM projects ORDER BY id";
/// Languages of a snapshot with files and lines of code, most lines first: snapshot.
pub const LANGUAGE_SHARES: &str = "SELECT language, COUNT(*), COALESCE(SUM(loc), 0) FROM files \
    WHERE snapshot_id = ?1 GROUP BY language ORDER BY SUM(loc) DESC, language";
/// Number of types of a snapshot: snapshot.
pub const TYPE_COUNT: &str = "SELECT COUNT(*) FROM type_summary WHERE snapshot_id = ?1";

/// Namespaces, modules and types of one kind with their member count: snapshot, kind.
pub const OUTLINE_OF_KIND: &str = "SELECT s.kind, s.id, s.name, s.qualified_name, s.parent_id, \
    (SELECT COUNT(*) FROM symbols m WHERE m.snapshot_id = s.snapshot_id AND m.parent_id = s.id) \
    FROM symbols s WHERE s.snapshot_id = ?1 AND s.kind = ?2";
/// Member table of a symbol: snapshot, symbol, complexity metric.
pub const MEMBER_ROWS: &str = "SELECT s.id, s.name, s.kind, s.visibility, s.signature, \
    s.start_line, s.end_line, \
    (SELECT x.value FROM metrics x WHERE x.snapshot_id = s.snapshot_id AND x.symbol_id = s.id \
        AND x.kind = ?3), \
    COALESCE((SELECT u.usages FROM usage_counts u WHERE u.snapshot_id = s.snapshot_id \
        AND u.symbol_id = s.id), 0) \
    FROM symbols s WHERE s.snapshot_id = ?1 AND s.parent_id = ?2 ORDER BY s.start_line, s.start_column, s.id";
