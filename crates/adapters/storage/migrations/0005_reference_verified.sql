-- Schema version 5: a reference says whether level 2 produced it (ADR 0032). Every reference that
-- exists so far comes from level 1 and is unchecked.

ALTER TABLE symbol_references ADD COLUMN verified INTEGER NOT NULL DEFAULT 0;

DROP VIEW snapshot_references;
CREATE VIEW snapshot_references AS
SELECT f.snapshot_id AS snapshot_id, r.id AS id, r.from_symbol AS from_symbol,
       r.to_symbol AS to_symbol, r.file_id AS file_id, r.start_line AS start_line,
       r.start_column AS start_column, r.end_line AS end_line, r.end_column AS end_column,
       r.kind AS kind, r.confidence AS confidence, r.verified AS verified
FROM files f
JOIN symbol_references r ON r.file_id = f.id
JOIN symbol_rows t ON t.id = r.to_symbol
JOIN files tf ON tf.snapshot_id = f.snapshot_id AND tf.id = t.file_id
WHERE r.first_snapshot <= f.snapshot_id;
