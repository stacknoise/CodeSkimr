-- Schema version 9: the time a full scan spent parsing the files of each language.
-- An incremental scan copies the rows of the previous snapshot, so the figure always comes from the
-- last full scan. A snapshot from before this version has no rows.

CREATE TABLE parse_times (
    snapshot_id INTEGER NOT NULL,
    language TEXT NOT NULL,
    micros INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, language)
) WITHOUT ROWID;
