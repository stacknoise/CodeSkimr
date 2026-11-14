-- Schema version 7: the result of checking the tools a project needs for level 2 (ADR 0033,
-- decision 8). One report per project, replaced by every check; its findings follow in order.

CREATE TABLE tool_reports (
    project_id INTEGER PRIMARY KEY REFERENCES projects (id) ON DELETE CASCADE,
    checked_at INTEGER NOT NULL
) STRICT;

CREATE TABLE tool_findings (
    project_id  INTEGER NOT NULL REFERENCES tool_reports (project_id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    language    TEXT NOT NULL,
    tool        TEXT NOT NULL,
    role        TEXT NOT NULL,
    state       TEXT NOT NULL,
    found       TEXT,
    required    TEXT,
    purpose_key TEXT NOT NULL,
    install_key TEXT NOT NULL,
    PRIMARY KEY (project_id, position)
) STRICT;
