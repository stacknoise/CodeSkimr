-- Schema version 17: what the last scan of a project says about itself (ADR 0072, decision 9).
--
-- Only language ids, counts, reason keys and error code ids are stored: no path, no tool name or
-- version text, no output of an indexer, no project name. The rows go with the project.

-- The newest scan of the project: when, at which level and from which channel. A scan replaces it.
CREATE TABLE scan_reports (
    project_id INTEGER PRIMARY KEY REFERENCES projects (id) ON DELETE CASCADE,
    scanned_at INTEGER NOT NULL,
    level TEXT NOT NULL,
    origin TEXT NOT NULL
);

-- The newest scan that ran the second pass. A level 1 scan of watch mode or MCP leaves it alone.
CREATE TABLE level2_reports (
    project_id INTEGER PRIMARY KEY REFERENCES projects (id) ON DELETE CASCADE,
    run_at INTEGER NOT NULL,
    origin TEXT NOT NULL
);

-- What the pass did per language: `checked`, `skipped` or `failed`. `files` and `stale` are set for
-- `checked`, `reason` for `skipped`, `code` for `failed`.
CREATE TABLE level2_languages (
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    outcome TEXT NOT NULL,
    files INTEGER,
    stale INTEGER,
    reason TEXT,
    code TEXT,
    PRIMARY KEY (project_id, language)
);
