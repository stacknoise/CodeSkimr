-- Schema version 8: how a snapshot named its symbols (ADR 0036).
-- A snapshot records the naming rules of the language packs it was scanned with (for example whether
-- the file is part of the qualified names). An incremental scan with other rules is a full scan, so a
-- project never mixes names of two rules. A snapshot from before this version has no record.

ALTER TABLE snapshots ADD COLUMN naming_rules TEXT;
