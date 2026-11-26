-- Schema version 25 (after 0023 and 0024): how long a scan took and how many files it changed (ADR 0097, issues #711 and #725).
--
-- `duration_ms` is the wall time of the scan that wrote the snapshot, from the start of the discover
-- stage to the commit of the snapshot. `files_changed` counts the files of that scan that were new,
-- changed or gone against the previous snapshot; the first scan of a project has none. A snapshot
-- written before this version has neither, and the screen shows a dash. A scan that finds nothing
-- changed (ADR 0093) writes no snapshot and leaves both as they are.
ALTER TABLE snapshots ADD COLUMN duration_ms INTEGER;
ALTER TABLE snapshots ADD COLUMN files_changed INTEGER;
