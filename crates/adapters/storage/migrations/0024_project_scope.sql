-- Schema version 24 (0023 belongs to #761): what a project leaves out of its scans (ADR 0102).
--
-- The scope lists what is left out, so a project without rows covers everything, which is what
-- every project did before. A folder is relative to the project folder, written with `/`; it
-- leaves out everything below it. The rows belong to the user's choice for this project and are
-- kept when the project is scanned again.

CREATE TABLE scope_languages (
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    PRIMARY KEY (project_id, language)
) WITHOUT ROWID;

CREATE TABLE scope_folders (
    project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    folder TEXT NOT NULL,
    PRIMARY KEY (project_id, folder)
) WITHOUT ROWID;
