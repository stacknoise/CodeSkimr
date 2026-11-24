-- Schema version 21: every file a scan left out for good (an encoding that is not read), with the
-- hash and language it had then and the code of the failure (issue #684, ADR 0093). `snapshot_left_out` names only the first
-- `scan.max_left_out_listed` files and no hash; these rows let the next scan see that such a file
-- is still the same, so that a scan without changes writes no snapshot. A file whose failure may
-- pass (a timeout, a locked file) has no row: the next scan tries it again. A snapshot from before
-- this version has no rows.

CREATE TABLE snapshot_skipped_files (
    snapshot_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    language TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    code TEXT NOT NULL,
    PRIMARY KEY (snapshot_id, path)
) WITHOUT ROWID;
