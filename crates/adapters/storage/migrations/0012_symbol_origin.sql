-- Schema version 11: a symbol has an origin (ADR 0055). `syntax` is a symbol that level 1 read from
-- the syntax tree, `index` one that level 2 created from what an index defines and level 1 does not
-- know. The rows follow their file like every shared row (ADR 0032). The number of this file is
-- the number the package was given; 0011 and 0012 belong to other open changes and take their
-- place in `MIGRATIONS` before this one when they are merged.
ALTER TABLE symbol_rows ADD COLUMN origin TEXT NOT NULL DEFAULT 'syntax';

DROP VIEW symbols;
CREATE VIEW symbols AS
SELECT f.snapshot_id AS snapshot_id, s.id AS id, s.file_id AS file_id, s.kind AS kind,
       s.name AS name, s.qualified_name AS qualified_name, s.parent_id AS parent_id,
       s.visibility AS visibility, s.is_static AS is_static, s.start_line AS start_line,
       s.start_column AS start_column, s.end_line AS end_line, s.end_column AS end_column,
       s.signature AS signature, s.doc AS doc, s.origin AS origin
FROM files f JOIN symbol_rows s ON s.file_id = f.id;
