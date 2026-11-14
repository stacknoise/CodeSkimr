-- Schema version 6: what the user decided about level 2 for a project (ADR 0033, decision 9).
-- A project without a row has neither enabled nor confirmed it. The record lives here, in the
-- user's own database, and never in the project's files.

CREATE TABLE level2_consent (
    project_id INTEGER PRIMARY KEY REFERENCES projects (id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL,
    confirmed INTEGER NOT NULL
);
