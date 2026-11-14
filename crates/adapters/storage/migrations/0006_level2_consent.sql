-- Schema version 6: what the user decided about level 2 for a project (ADR 0033, decision 9).
-- A project without a row has neither enabled nor confirmed it. The record lives here, in the
-- user's own database, and never in the project's files. The confirmation is bound to the source
-- the project had when the user confirmed (kind, location, branch): a project that is pointed at
-- another repository or folder is unconfirmed again.

CREATE TABLE level2_consent (
    project_id INTEGER PRIMARY KEY REFERENCES projects (id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL,
    confirmed INTEGER NOT NULL,
    source_kind TEXT NOT NULL,
    location TEXT NOT NULL,
    branch TEXT NOT NULL
);
