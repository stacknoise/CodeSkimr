-- Schema version 13: the warnings of a scan, one row per code with its count (issue #433).
-- The count covers the whole snapshot, so an incremental scan writes its own rows and carries
-- nothing over. A snapshot from before this version has no rows.

CREATE TABLE snapshot_warnings (
    snapshot_id INTEGER NOT NULL,
    code TEXT NOT NULL,
    count INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, code)
) WITHOUT ROWID;
