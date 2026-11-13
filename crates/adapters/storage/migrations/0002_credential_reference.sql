-- Schema version 2: the name of the credential a Git source logs in with (blueprint 12.2, point 4).
-- Only the name is stored, never a secret; the empty text means "no credential".

ALTER TABLE projects ADD COLUMN credential TEXT NOT NULL DEFAULT '';
