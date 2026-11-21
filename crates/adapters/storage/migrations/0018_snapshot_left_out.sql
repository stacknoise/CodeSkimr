-- Schema version 18: the files a scan left out because one file alone could not be analysed
-- (issue #635, ADR 0085). The scan names at most `scan.max_left_out_listed` of them; the count per
-- code of all of them stays in `snapshot_warnings`. An incremental scan writes its own rows and
-- carries nothing over, like the warnings. A snapshot from before this version has no rows.

CREATE TABLE snapshot_left_out (
    snapshot_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    code TEXT NOT NULL,
    kept INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, path)
) WITHOUT ROWID;
