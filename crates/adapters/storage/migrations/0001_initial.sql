-- Schema version 1: registered projects, snapshots with their facts, and the read models.
-- Facts are keyed by (snapshot_id, id): an incremental snapshot carries unchanged rows over
-- with their ids, so the same id can occur in several snapshots.

CREATE TABLE projects (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    location    TEXT NOT NULL,
    branch      TEXT NOT NULL,
    UNIQUE (source_kind, location, branch)
) STRICT;

CREATE TABLE layers (
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    position   INTEGER NOT NULL,
    name       TEXT NOT NULL,
    PRIMARY KEY (project_id, position)
) STRICT;

CREATE TABLE layer_namespaces (
    project_id     INTEGER NOT NULL,
    layer_position INTEGER NOT NULL,
    position       INTEGER NOT NULL,
    namespace      TEXT NOT NULL,
    PRIMARY KEY (project_id, layer_position, position),
    FOREIGN KEY (project_id, layer_position) REFERENCES layers (project_id, position)
        ON DELETE CASCADE
) STRICT;

CREATE TABLE snapshots (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    commit_hash TEXT,
    created    INTEGER NOT NULL,
    mode       TEXT NOT NULL,
    files      INTEGER NOT NULL,
    symbols    INTEGER NOT NULL,
    refs       INTEGER NOT NULL
) STRICT;

CREATE INDEX snapshots_by_project ON snapshots (project_id, id);

CREATE TABLE files (
    snapshot_id  INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    id           INTEGER NOT NULL,
    path         TEXT NOT NULL,
    language     TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    loc          INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, id),
    UNIQUE (snapshot_id, path)
) STRICT;

CREATE TABLE symbols (
    snapshot_id    INTEGER NOT NULL,
    id             INTEGER NOT NULL,
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
    doc            TEXT,
    PRIMARY KEY (snapshot_id, id),
    FOREIGN KEY (snapshot_id, file_id) REFERENCES files (snapshot_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX symbols_by_file ON symbols (snapshot_id, file_id);
CREATE INDEX symbols_by_parent ON symbols (snapshot_id, parent_id);

CREATE TABLE symbol_references (
    snapshot_id  INTEGER NOT NULL,
    id           INTEGER NOT NULL,
    from_symbol  INTEGER NOT NULL,
    to_symbol    INTEGER NOT NULL,
    file_id      INTEGER NOT NULL,
    start_line   INTEGER NOT NULL,
    start_column INTEGER NOT NULL,
    end_line     INTEGER NOT NULL,
    end_column   INTEGER NOT NULL,
    kind         TEXT NOT NULL,
    confidence   TEXT NOT NULL,
    PRIMARY KEY (snapshot_id, id),
    FOREIGN KEY (snapshot_id, file_id) REFERENCES files (snapshot_id, id) ON DELETE CASCADE,
    FOREIGN KEY (snapshot_id, from_symbol) REFERENCES symbols (snapshot_id, id) ON DELETE CASCADE,
    FOREIGN KEY (snapshot_id, to_symbol) REFERENCES symbols (snapshot_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX references_by_target ON symbol_references (snapshot_id, to_symbol);
CREATE INDEX references_by_source ON symbol_references (snapshot_id, from_symbol);

CREATE TABLE metrics (
    snapshot_id INTEGER NOT NULL,
    symbol_id   INTEGER NOT NULL,
    kind        TEXT NOT NULL,
    value       REAL NOT NULL,
    PRIMARY KEY (snapshot_id, symbol_id, kind),
    FOREIGN KEY (snapshot_id, symbol_id) REFERENCES symbols (snapshot_id, id) ON DELETE CASCADE
) STRICT;

-- Read models, rebuilt for every new snapshot (blueprint 6.2).

CREATE TABLE search_index (
    snapshot_id    INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    symbol_id      INTEGER NOT NULL,
    name_lower     TEXT NOT NULL,
    name           TEXT NOT NULL,
    qualified_name TEXT NOT NULL,
    kind           TEXT NOT NULL,
    path           TEXT NOT NULL,
    PRIMARY KEY (snapshot_id, symbol_id)
) STRICT;

CREATE INDEX search_by_name ON search_index (snapshot_id, name_lower);

CREATE TABLE usage_counts (
    snapshot_id INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    symbol_id   INTEGER NOT NULL,
    usages      INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, symbol_id)
) STRICT;

CREATE TABLE layer_graph (
    snapshot_id       INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    from_layer        TEXT NOT NULL,
    to_layer          TEXT NOT NULL,
    edge_count        INTEGER NOT NULL,
    violates_layering INTEGER NOT NULL,
    in_cycle          INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, from_layer, to_layer)
) STRICT;

CREATE TABLE type_summary (
    snapshot_id    INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    symbol_id      INTEGER NOT NULL,
    member_count   INTEGER NOT NULL,
    loc            REAL,
    max_complexity REAL,
    maintainability REAL,
    PRIMARY KEY (snapshot_id, symbol_id)
) STRICT;
