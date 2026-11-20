-- Schema version 11: the layers a layer may depend on (ADR 0054).
-- `restricted` is 0 for a layer without a limit (no `may_use`), 1 when `layer_uses` lists all
-- allowed layers (an empty list then allows none). Rows from before this version stay unlimited.

ALTER TABLE layers ADD COLUMN restricted INTEGER NOT NULL DEFAULT 0;

CREATE TABLE layer_uses (
    project_id     INTEGER NOT NULL,
    layer_position INTEGER NOT NULL,
    position       INTEGER NOT NULL,
    used           TEXT NOT NULL,
    PRIMARY KEY (project_id, layer_position, position),
    FOREIGN KEY (project_id, layer_position) REFERENCES layers (project_id, position)
        ON DELETE CASCADE
) STRICT;
