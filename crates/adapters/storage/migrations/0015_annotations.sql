-- Schema version 15: annotations on symbols, their baselines and the alert list (ADR 0068).
-- They belong to the project and not to a snapshot: no table here names a snapshot as its owner, so a
-- new scan and the retention of snapshots never touch them, and deleting a project deletes them.
-- A symbol is found again by its target key (kind, qualified name, parameter types), never by an id of
-- a snapshot. The parameter types are joined with the unit separator (U+001F), empty for none.

CREATE TABLE annotation (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id    INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    target_kind   TEXT NOT NULL,
    target_name   TEXT NOT NULL,
    target_params TEXT NOT NULL,
    kind          TEXT NOT NULL,
    title         TEXT,
    text          TEXT,
    status        TEXT,
    due           INTEGER,
    assignee      TEXT,
    author        TEXT NOT NULL,
    created       INTEGER NOT NULL,
    updated       INTEGER NOT NULL
) STRICT;

-- List by project (and kind), and list by symbol.
CREATE INDEX annotation_by_project ON annotation (project_id, kind, id);
CREATE INDEX annotation_by_target ON annotation (project_id, target_name, target_kind, target_params);
-- A symbol has at most one critical annotation; the store checks it first, the index guards it.
CREATE UNIQUE INDEX annotation_one_critical
    ON annotation (project_id, target_name, target_kind, target_params) WHERE kind = 'critical';

-- What a critical symbol looked like when the user last accepted it.
CREATE TABLE annotation_baseline (
    annotation_id  INTEGER PRIMARY KEY REFERENCES annotation (id) ON DELETE CASCADE,
    content_hash   TEXT NOT NULL,
    signature_hash TEXT NOT NULL,
    loc            INTEGER NOT NULL,
    cc             INTEGER NOT NULL
) STRICT;

-- `snapshot_id` and `commit_hash` only say where the latest sighting was made. There is no foreign
-- key on the snapshot: it may be pruned long before the alert is dismissed.
CREATE TABLE alert (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    annotation_id INTEGER NOT NULL REFERENCES annotation (id) ON DELETE CASCADE,
    kind          TEXT NOT NULL,
    state         TEXT NOT NULL,
    counter       INTEGER NOT NULL,
    first_seen    INTEGER NOT NULL,
    last_seen     INTEGER NOT NULL,
    dismissed     INTEGER,
    snapshot_id   INTEGER NOT NULL,
    commit_hash   TEXT,
    trigger       TEXT NOT NULL,
    changes       TEXT NOT NULL,
    refs          INTEGER,
    closest       TEXT
) STRICT;

-- At most one open alert per annotation; a further sighting raises its counter.
CREATE UNIQUE INDEX alert_one_open ON alert (annotation_id) WHERE state = 'open';
-- List by state, newest first, and the purge of old dismissed alerts.
CREATE INDEX alert_by_state ON alert (state, last_seen);
CREATE INDEX alert_by_dismissal ON alert (state, dismissed);
