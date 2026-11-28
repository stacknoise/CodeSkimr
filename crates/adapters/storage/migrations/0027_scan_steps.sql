-- Schema version 27: how long each step of writing a snapshot took, for every scan that wrote one.
--
-- The write of a snapshot has steps (`PersistStep`), and on a large project one of them takes
-- minutes. A row here is the time one step took in the scan of `scan_log`; a scan that found
-- nothing changed wrote nothing and has no rows. Rows go with their scan (`ON DELETE CASCADE`).
-- The scans that are already stored have none.
CREATE TABLE scan_steps (
    scan_id     INTEGER NOT NULL REFERENCES scan_log (id) ON DELETE CASCADE,
    step        TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    PRIMARY KEY (scan_id, step)
) STRICT, WITHOUT ROWID;
