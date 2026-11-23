-- Schema version 20: the statistics of a snapshot are counted when the snapshot is written, not
-- when the statistics screen opens (ADR 0091).
--
-- One row per type and callable of a snapshot with what the rankings sort and filter by: the
-- usage counts, the measured values, whether the symbol is test code, and `place`, the position
-- of the symbol in the order of path, qualified name and id, so that equal values come in the same
-- order every time. A snapshot written before this version has no rows; the statistics screen
-- counts them when it first opens the project (`snapshot_statistics` says whether they exist).

CREATE TABLE symbol_statistics (
    snapshot_id INTEGER NOT NULL,
    symbol_id INTEGER NOT NULL,
    place INTEGER NOT NULL,
    kind TEXT NOT NULL,
    is_type INTEGER NOT NULL,
    language TEXT NOT NULL,
    visibility TEXT,
    test INTEGER NOT NULL,
    overrides INTEGER NOT NULL,
    indirect INTEGER NOT NULL,
    certain INTEGER NOT NULL,
    total INTEGER NOT NULL,
    complexity REAL,
    loc REAL,
    maintainability REAL,
    efferent REAL,
    afferent REAL,
    PRIMARY KEY (snapshot_id, symbol_id)
) WITHOUT ROWID;

-- What the rows were counted for: the names of the test folders that marked the test code, and
-- the usages of all symbols of the snapshot (also those the rankings do not hold).
CREATE TABLE snapshot_statistics (
    snapshot_id INTEGER PRIMARY KEY,
    test_key TEXT NOT NULL,
    certain_references INTEGER NOT NULL,
    probable_references INTEGER NOT NULL
);

-- The first rows of the rankings that the overview shows (the most used by their certain usages,
-- the most complex, the most coupled, and the unused ones), in the order of the ranking, with the
-- columns the rankings filter by, so that a question reads the first rows of an index and nothing
-- else, however many symbols there are. The ranking by all usages and the one by size are only
-- asked for on their tabs and read the rows of the snapshot once.
CREATE INDEX statistics_by_certain ON symbol_statistics
    (snapshot_id, certain DESC, total DESC, place, is_type, test, language) WHERE total > 0;
CREATE INDEX statistics_by_complexity ON symbol_statistics
    (snapshot_id, complexity DESC, place, is_type, test, language)
    WHERE is_type = 0 AND complexity IS NOT NULL;
CREATE INDEX statistics_by_efferent ON symbol_statistics
    (snapshot_id, efferent DESC, place, is_type, test, language) WHERE efferent IS NOT NULL;
-- The symbols without usages in the order of their place; the assessment of one is read from its
-- visibility and language.
CREATE INDEX statistics_by_place ON symbol_statistics
    (snapshot_id, place, is_type, test, language, visibility) WHERE total = 0 AND indirect = 0;

-- How many rows of a snapshot there are per group of what the rankings filter and hold: so the
-- counts of the overview and of the tabs need no pass over the rows.
CREATE TABLE statistics_facets (
    snapshot_id INTEGER NOT NULL,
    is_type INTEGER NOT NULL,
    language TEXT NOT NULL,
    visibility TEXT,
    test INTEGER NOT NULL,
    used INTEGER NOT NULL,
    idle INTEGER NOT NULL,
    has_complexity INTEGER NOT NULL,
    has_loc INTEGER NOT NULL,
    has_efferent INTEGER NOT NULL,
    n INTEGER NOT NULL
);
CREATE INDEX statistics_facets_by_snapshot ON statistics_facets (snapshot_id);

-- How many callables have each cyclomatic complexity, for the histogram.
CREATE TABLE statistics_complexity (
    snapshot_id INTEGER NOT NULL,
    test INTEGER NOT NULL,
    complexity REAL NOT NULL,
    n INTEGER NOT NULL
);
CREATE INDEX statistics_complexity_by_snapshot ON statistics_complexity (snapshot_id);
