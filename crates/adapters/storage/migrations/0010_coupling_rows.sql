-- Schema version 10: coupling metrics (ce, ca, instability) depend on references across files, so
-- they are stored per snapshot and not shared like the other metrics (ADR 0037, issue #150).
CREATE TABLE coupling_rows (
    snapshot_id INTEGER NOT NULL,
    symbol_id   INTEGER NOT NULL,
    kind        TEXT NOT NULL,
    value       REAL NOT NULL,
    PRIMARY KEY (snapshot_id, symbol_id, kind)
) STRICT, WITHOUT ROWID;

-- The metrics of a snapshot: the shared ones of its symbols and its own coupling rows.
DROP VIEW metrics;
CREATE VIEW metrics AS
SELECT f.snapshot_id AS snapshot_id, m.symbol_id AS symbol_id, m.kind AS kind, m.value AS value
FROM files f
JOIN symbol_rows s ON s.file_id = f.id
JOIN metric_rows m ON m.symbol_id = s.id
UNION ALL
SELECT snapshot_id, symbol_id, kind, value FROM coupling_rows;
