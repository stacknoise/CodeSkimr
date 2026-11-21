-- Schema version 18: a snapshot remembers how many of its places match several symbols (ADR 0085).
--
-- The next incremental scan starts from this number and reads only the references of the files it
-- touches. NULL means that the number is not known: for snapshots written before this version and
-- after level 2 replaced references of the snapshot. The scan then reads every reference again.

ALTER TABLE snapshots ADD COLUMN ambiguous_places INTEGER;
