-- Schema version 11 in this branch (file number 0012: 0011 is taken by the layer directions of
-- PR 339): the names as written (issue 49).
--
-- An incremental scan resolves every name of the unchanged files again against the changed symbol
-- table, so the result equals a full scan: references to a symbol that was renamed or moved are
-- found again, a name that matched nothing finds a target that a changed file now declares, and a
-- reference gets the confidence of the new situation. For that the store keeps the names of each
-- file as the analysis found them, resolved or not. A name belongs to its file like a symbol does,
-- so it is shared by the snapshots that carry the file (ADR 0032) and removed with the last of them.
-- A snapshot written before this version does not hold them; the next scan after it is a full scan.

ALTER TABLE snapshots ADD COLUMN names_stored INTEGER NOT NULL DEFAULT 0;

CREATE TABLE pending_references (
    file_id       INTEGER NOT NULL,
    from_symbol   INTEGER NOT NULL,
    start_line    INTEGER NOT NULL,
    start_column  INTEGER NOT NULL,
    end_line      INTEGER NOT NULL,
    end_column    INTEGER NOT NULL,
    kind          TEXT NOT NULL,
    target        TEXT NOT NULL,
    receiver_kind TEXT,
    receiver_name TEXT
) STRICT;

CREATE INDEX pending_references_by_file ON pending_references (file_id);
