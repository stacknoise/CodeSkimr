-- Schema version 4: symbols, metrics, references and fingerprints are stored once and shared by the
-- snapshots that contain them (ADR 0032), so an incremental scan copies only the file rows
-- (issue #108).
--
-- A symbol belongs to its file, a metric and a fingerprint to their symbol, a reference to the
-- file it stands in. Ids are never reused, so a file keeps its symbols for as long as it is carried
-- from snapshot to snapshot. A snapshot contains the symbols of its files (views `symbols` and
-- `metrics`, which look like the former tables) and the references whose file and target are in it
-- (view `snapshot_references`). The tables lose their foreign keys: the rows of a snapshot are removed by the prune of the
-- store, which keeps the rows that another snapshot still contains.

CREATE TABLE symbol_references_shared (
    id             INTEGER PRIMARY KEY,
    first_snapshot INTEGER NOT NULL,
    from_symbol  INTEGER NOT NULL,
    to_symbol    INTEGER NOT NULL,
    file_id      INTEGER NOT NULL,
    start_line   INTEGER NOT NULL,
    start_column INTEGER NOT NULL,
    end_line     INTEGER NOT NULL,
    end_column   INTEGER NOT NULL,
    kind         TEXT NOT NULL,
    confidence   TEXT NOT NULL
) STRICT;

-- Snapshots copied their references with the same ids, so one row per id is the whole content. The
-- first snapshot that held it is the first one that sees it: later snapshots carry it on, earlier
-- ones never had it.
INSERT INTO symbol_references_shared
    (id, first_snapshot, from_symbol, to_symbol, file_id, start_line, start_column, end_line,
     end_column, kind, confidence)
SELECT id, MIN(snapshot_id), from_symbol, to_symbol, file_id, start_line, start_column, end_line,
       end_column, kind, confidence
FROM symbol_references GROUP BY id;

DROP TABLE symbol_references;
ALTER TABLE symbol_references_shared RENAME TO symbol_references;

CREATE INDEX references_by_target ON symbol_references (to_symbol);
CREATE INDEX references_by_source ON symbol_references (from_symbol);
CREATE INDEX references_by_file ON symbol_references (file_id);

CREATE TABLE fingerprints_shared (
    symbol_id INTEGER NOT NULL,
    position  INTEGER NOT NULL,
    hash      INTEGER NOT NULL,
    PRIMARY KEY (symbol_id, position)
) STRICT, WITHOUT ROWID;

INSERT OR IGNORE INTO fingerprints_shared (symbol_id, position, hash)
SELECT symbol_id, position, hash FROM fingerprints;

DROP TABLE fingerprints;
ALTER TABLE fingerprints_shared RENAME TO fingerprints;

CREATE INDEX fingerprints_by_hash ON fingerprints (hash);


-- Symbols and metrics, one row per id instead of one per snapshot.
CREATE TABLE symbol_rows (
    id             INTEGER PRIMARY KEY,
    file_id        INTEGER NOT NULL,
    kind           TEXT NOT NULL,
    name           TEXT NOT NULL,
    qualified_name TEXT NOT NULL,
    parent_id      INTEGER,
    visibility     TEXT,
    is_static      INTEGER NOT NULL,
    start_line     INTEGER NOT NULL,
    start_column   INTEGER NOT NULL,
    end_line       INTEGER NOT NULL,
    end_column     INTEGER NOT NULL,
    signature      TEXT,
    doc            TEXT
) STRICT;

INSERT OR IGNORE INTO symbol_rows
    (id, file_id, kind, name, qualified_name, parent_id, visibility, is_static, start_line,
     start_column, end_line, end_column, signature, doc)
SELECT id, file_id, kind, name, qualified_name, parent_id, visibility, is_static, start_line,
       start_column, end_line, end_column, signature, doc
FROM symbols;

CREATE TABLE metric_rows (
    symbol_id INTEGER NOT NULL,
    kind      TEXT NOT NULL,
    value     REAL NOT NULL,
    PRIMARY KEY (symbol_id, kind)
) STRICT, WITHOUT ROWID;

INSERT OR REPLACE INTO metric_rows (symbol_id, kind, value)
SELECT symbol_id, kind, value FROM metrics ORDER BY snapshot_id;

DROP TABLE metrics;
DROP TABLE symbols;

CREATE INDEX symbols_by_file ON symbol_rows (file_id);
CREATE INDEX symbols_by_parent ON symbol_rows (parent_id);
CREATE INDEX symbols_by_kind ON symbol_rows (kind);

-- The prune looks for the files that no other snapshot contains.
CREATE INDEX files_by_id ON files (id);

-- The symbols and metrics a snapshot contains, with the columns of the former tables.
CREATE VIEW symbols AS
SELECT f.snapshot_id AS snapshot_id, s.id AS id, s.file_id AS file_id, s.kind AS kind,
       s.name AS name, s.qualified_name AS qualified_name, s.parent_id AS parent_id,
       s.visibility AS visibility, s.is_static AS is_static, s.start_line AS start_line,
       s.start_column AS start_column, s.end_line AS end_line, s.end_column AS end_column,
       s.signature AS signature, s.doc AS doc
FROM files f JOIN symbol_rows s ON s.file_id = f.id;

CREATE VIEW metrics AS
SELECT f.snapshot_id AS snapshot_id, m.symbol_id AS symbol_id, m.kind AS kind, m.value AS value
FROM files f
JOIN symbol_rows s ON s.file_id = f.id
JOIN metric_rows m ON m.symbol_id = s.id;

-- The references a snapshot contains: the file they stand in and the symbol they point to are in
-- it, and it is not older than the scan that wrote the reference.
CREATE VIEW snapshot_references AS
SELECT f.snapshot_id AS snapshot_id, r.id AS id, r.from_symbol AS from_symbol,
       r.to_symbol AS to_symbol, r.file_id AS file_id, r.start_line AS start_line,
       r.start_column AS start_column, r.end_line AS end_line, r.end_column AS end_column,
       r.kind AS kind, r.confidence AS confidence
FROM files f
JOIN symbol_references r ON r.file_id = f.id
JOIN symbol_rows t ON t.id = r.to_symbol
JOIN files tf ON tf.snapshot_id = f.snapshot_id AND tf.id = t.file_id
WHERE r.first_snapshot <= f.snapshot_id;
