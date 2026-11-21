-- Schema version 19: a snapshot remembers how many of its places match several symbols (ADR 0086).
--
-- The next incremental scan starts from this number and reads only the references of the files it
-- touches. NULL means that the number is not known: for snapshots written before this version and
-- after level 2 replaced references of the snapshot. The scan then reads every reference again.

ALTER TABLE snapshots ADD COLUMN ambiguous_places INTEGER;

-- The files that hold a reference verified by level 2 are read at the start of every incremental
-- scan. Only verified rows are in the index, so level 1 pays nothing for it.
CREATE INDEX references_verified ON symbol_references (file_id) WHERE verified = 1;
