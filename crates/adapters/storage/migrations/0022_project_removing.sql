-- Schema version 22: a project whose removal has begun (issue #714).
--
-- The removal runs in many short writes, so that other writes get their turn. It marks the project
-- first; a marked project is no project any more for scans, renames and reads, and only the
-- removal itself, which can be started again after an interruption, works on it.

ALTER TABLE projects ADD COLUMN removing INTEGER NOT NULL DEFAULT 0;
