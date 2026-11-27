-- Schema version 26: every scan is one row of the history, also one that found nothing changed
-- (ADR 0093, addendum of 2026-11-27, and ADR 0097).
--
-- A scan that finds nothing changed writes no snapshot; it confirms the newest one. Until this
-- version that only moved the time of the snapshot, so three scans in a row showed one row. Now
-- every scan has a row here: a scan that wrote a snapshot names that snapshot, a scan that found
-- nothing changed names the snapshot it confirmed (so several rows can share one snapshot).
-- Rows go with their snapshot (`ON DELETE CASCADE`), and the writer keeps at most
-- `snapshots.retention_count` confirmations per snapshot besides the row of the scan that wrote it.
CREATE TABLE scan_log (
    id            INTEGER PRIMARY KEY,
    snapshot_id   INTEGER NOT NULL REFERENCES snapshots (id) ON DELETE CASCADE,
    created       INTEGER NOT NULL,
    commit_hash   TEXT,
    mode          TEXT NOT NULL,
    duration_ms   INTEGER,
    files_changed INTEGER
) STRICT;

CREATE INDEX scan_log_by_snapshot ON scan_log (snapshot_id, id);

-- The scans that are already stored: one row per snapshot, with what the snapshot holds.
INSERT INTO scan_log (snapshot_id, created, commit_hash, mode, duration_ms, files_changed)
    SELECT id, created, commit_hash, mode, duration_ms, files_changed FROM snapshots ORDER BY id;
