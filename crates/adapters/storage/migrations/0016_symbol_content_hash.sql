-- Schema version 16: the content hash of a symbol and what the alert step needs to remember (ADR 0068).
--
-- `content_hash` is the hash over the token stream of a type or member, taken in the parse stage. The
-- row belongs to its file like every symbol row (ADR 0032), so an incremental scan keeps the hashes of
-- the files it does not analyse again. A namespace, a module, a parameter and a symbol that level 2
-- created have none. A snapshot written before this version has none for any symbol; the naming rules
-- of the packs changed with it, so the next scan after the update is a full one.
ALTER TABLE symbol_rows ADD COLUMN content_hash TEXT;

-- Whether the newest scan did not find the symbol of an annotation. It is set for every kind; only a
-- critical annotation raises an alert for it.
ALTER TABLE annotation ADD COLUMN without_target INTEGER NOT NULL DEFAULT 0;

-- What the symbol looked like at the latest sighting of an alert of kind `changed`. A scan raises the
-- counter of the open alert only if the symbol looks different from that.
ALTER TABLE alert ADD COLUMN seen_content TEXT;
ALTER TABLE alert ADD COLUMN seen_signature TEXT;
ALTER TABLE alert ADD COLUMN seen_loc INTEGER;
ALTER TABLE alert ADD COLUMN seen_cc INTEGER;
