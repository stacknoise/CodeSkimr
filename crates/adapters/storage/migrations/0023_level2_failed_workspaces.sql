-- Schema version 23: workspaces of an indexer that failed while others ran through (ADR 0100).
--
-- Only a count is stored, set for a `checked` language line. Lines of earlier scans have none (0).

ALTER TABLE level2_languages ADD COLUMN failed_workspaces INTEGER NOT NULL DEFAULT 0;
