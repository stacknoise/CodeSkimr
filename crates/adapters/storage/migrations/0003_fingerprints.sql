-- Schema version 3: the syntax fingerprints of the callable symbols (blueprint 10.4), so that
-- `find_similar_code` can compare code with everything in the project, not only with the files
-- of one call. A hash is the 64 bits of the fingerprint stored as a signed integer.

CREATE TABLE fingerprints (
    snapshot_id INTEGER NOT NULL,
    symbol_id   INTEGER NOT NULL,
    position    INTEGER NOT NULL,
    hash        INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, symbol_id, position),
    FOREIGN KEY (snapshot_id, symbol_id) REFERENCES symbols (snapshot_id, id) ON DELETE CASCADE
) STRICT;

CREATE INDEX fingerprints_by_hash ON fingerprints (snapshot_id, hash);
