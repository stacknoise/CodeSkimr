//! Fixed values of the storage adapter. SQL statements live in `sql`.

/// Schema migrations in order; the position plus one is the schema version.
pub const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/0001_initial.sql"),
    include_str!("../migrations/0002_credential_reference.sql"),
    include_str!("../migrations/0003_fingerprints.sql"),
    include_str!("../migrations/0004_shared_rows.sql"),
    include_str!("../migrations/0005_reference_verified.sql"),
    include_str!("../migrations/0006_level2_consent.sql"),
    include_str!("../migrations/0007_tool_reports.sql"),
    include_str!("../migrations/0008_naming_rules.sql"),
    include_str!("../migrations/0009_parse_times.sql"),
    include_str!("../migrations/0010_coupling_rows.sql"),
    include_str!("../migrations/0011_pending_references.sql"),
    include_str!("../migrations/0012_layer_directions.sql"),
    include_str!("../migrations/0013_scan_warnings.sql"),
    include_str!("../migrations/0014_symbol_origin.sql"),
    include_str!("../migrations/0015_annotations.sql"),
    include_str!("../migrations/0016_symbol_content_hash.sql"),
    include_str!("../migrations/0017_scan_reports.sql"),
    include_str!("../migrations/0018_snapshot_left_out.sql"),
    include_str!("../migrations/0019_ambiguous_places.sql"),
    include_str!("../migrations/0020_symbol_statistics.sql"),
    include_str!("../migrations/0021_snapshot_skipped_files.sql"),
    include_str!("../migrations/0022_project_removing.sql"),
    include_str!("../migrations/0023_level2_failed_workspaces.sql"),
    include_str!("../migrations/0024_project_scope.sql"),
    include_str!("../migrations/0025_scan_history.sql"),
];

/// Name of the pragma that holds the schema version.
pub const PRAGMA_USER_VERSION: &str = "user_version";
/// Name of the pragma that selects the journal mode.
pub const PRAGMA_JOURNAL_MODE: &str = "journal_mode";
/// Write-ahead logging: readers do not block the single writer (blueprint 6.2).
pub const JOURNAL_MODE_WAL: &str = "WAL";
/// Name of the pragma that turns on foreign key checks.
pub const PRAGMA_FOREIGN_KEYS: &str = "foreign_keys";
/// Name of the pragma that sets the durability level.
pub const PRAGMA_SYNCHRONOUS: &str = "synchronous";
/// With WAL, `NORMAL` is durable enough and much faster than `FULL`.
pub const SYNCHRONOUS_NORMAL: &str = "NORMAL";
/// Name of the pragma that sets the page cache size.
pub const PRAGMA_CACHE_SIZE: &str = "cache_size";
/// Page cache of 64 MiB (a negative value is a size in KiB): a scan writes many rows and their
/// indexes in one transaction, and the default cache of 2 MiB spills to the file.
pub const CACHE_SIZE_KIB: i64 = -65_536;
/// Name of the pragma that selects where temporary tables and indexes live.
pub const PRAGMA_TEMP_STORE: &str = "temp_store";
/// Temporary structures of a statement stay in memory.
pub const TEMP_STORE_MEMORY: i64 = 2;
/// How long a connection waits for a lock before reporting a conflict.
pub const BUSY_TIMEOUT_MS: u64 = 5_000;
/// Name of the pragma that changes when another connection commits.
pub const PRAGMA_DATA_VERSION: &str = "data_version";
/// Name of the pragma that reports how many pages the database file has.
pub const PRAGMA_PAGE_COUNT: &str = "page_count";
/// Name of the pragma that caps how many pages the database file may have.
pub const PRAGMA_MAX_PAGE_COUNT: &str = "max_page_count";
/// How often switching a new file to the write-ahead log is tried again while another connection
/// holds the file. `SQLite` reports that case at once, without calling the busy handler.
pub const WAL_SWITCH_ATTEMPTS: u32 = 100;
/// Pause between two tries to switch to the write-ahead log.
pub const WAL_SWITCH_PAUSE_MS: u64 = 20;

/// Stored form of `ProjectSource::Folder`.
pub const SOURCE_FOLDER: &str = "folder";
/// Stored form of `ProjectSource::Git`.
pub const SOURCE_GIT: &str = "git";
/// Stored branch of a folder source, which has none.
pub const NO_BRANCH: &str = "";
/// Stored credential name of a source that uses none (a folder, or Git with the system's helper).
pub const NO_CREDENTIAL: &str = "";

/// Escapes `LIKE` wildcards in search queries.
pub const LIKE_ESCAPE: char = '\\';
/// `LIKE` wildcard for any run of characters.
pub const LIKE_ANY: char = '%';
/// `LIKE` wildcard for exactly one character.
pub const LIKE_ONE: char = '_';

/// The first id of an empty database.
pub const FIRST_ID: u64 = 1;
/// Distance between the highest stored id and the next free one.
pub const ID_STEP: u64 = 1;
/// Id of the first project of a fresh database.
pub const FIRST_PROJECT_ID: u64 = 1;
/// Position of the first layer, namespace or migration in their lists.
pub const FIRST_POSITION: i64 = 0;
/// Version of the first schema migration.
pub const FIRST_VERSION: i64 = 1;

/// Index of the first column of a row.
pub const FIRST_COLUMN: usize = 0;
/// Distance between two neighbouring columns.
pub const COLUMN_STEP: usize = 1;
/// Kinds that get an entry in `type_summary`.
pub const TYPE_KINDS: &[codeskimr_domain::SymbolKind] = &[
    codeskimr_domain::SymbolKind::Class,
    codeskimr_domain::SymbolKind::Interface,
    codeskimr_domain::SymbolKind::Trait,
    codeskimr_domain::SymbolKind::Record,
    codeskimr_domain::SymbolKind::Struct,
    codeskimr_domain::SymbolKind::Enum,
    codeskimr_domain::SymbolKind::Union,
];
/// Checks foreign keys at commit instead of per statement, so new rows may refer to carried ones.
pub const PRAGMA_DEFER_FOREIGN_KEYS: &str = "defer_foreign_keys";
/// Value that turns a boolean pragma on.
pub const PRAGMA_ON: bool = true;
/// Weight of one reference in the layer graph.
pub const EDGE_STEP: u32 = 1;

/// Bits of a fingerprint hash; the column holds them as a signed integer of the same width.
pub const HASH_BYTES: usize = 8;
/// A percentage is this many percent points of the whole.
pub const PERCENT_BASE: u64 = 100;
/// Opens the JSON list that carries the values of an `IN` condition.
pub const JSON_LIST_OPEN: char = '[';
/// Closes the JSON list.
pub const JSON_LIST_CLOSE: char = ']';
/// Separates the values of the JSON list.
pub const JSON_LIST_SEPARATOR: char = ',';
/// Around a text in a JSON list.
pub const JSON_TEXT_QUOTE: char = '"';
/// Containers that appear in the explorer tree next to the types of `TYPE_KINDS`.
pub const CONTAINER_KINDS: &[codeskimr_domain::SymbolKind] = &[
    codeskimr_domain::SymbolKind::Namespace,
    codeskimr_domain::SymbolKind::Module,
];

/// Stored label of `Receiver::Typed`.
pub const RECEIVER_TYPED: &str = "typed";
/// Stored label of `Receiver::Named`.
pub const RECEIVER_NAMED: &str = "named";
/// Stored label of `Receiver::Own`.
pub const RECEIVER_OWN: &str = "own";
/// Stored label of `Receiver::Base`.
pub const RECEIVER_BASE: &str = "base";
/// Stored label of `Receiver::Unknown`.
pub const RECEIVER_UNKNOWN: &str = "unknown";
/// Stored label of `Receiver::Free`.
pub const RECEIVER_FREE: &str = "free";

/// Below this many new references the indexes stay in place while rows are added.
pub const BULK_REFERENCES_MIN: usize = 20_000;
/// The indexes are built again after the load when the new references are at least this share of
/// the stored ones, expressed as "one in N" (a load of half the table or more).
pub const BULK_REFERENCES_SHARE_DIVISOR: usize = 2;

/// Position of the kind in a row of `REFERENCE_TARGETS`.
pub const REFERENCE_KIND_COLUMN: usize = 2;

/// Below this many new fingerprints the index by hash stays in place while rows are added.
pub const BULK_FINGERPRINTS_MIN: usize = 200_000;

/// Separates names in a text that is handed to SQL as a list of names (the unit separator, which
/// no name holds).
pub const NAME_SEPARATOR: char = '\u{1f}';

/// Seconds in a day, to turn the retention of dismissed alerts into a time.
pub const SECONDS_PER_DAY: i64 = 86_400;
/// Separates the labels of the changed parts of a symbol in the column `alert.changes`.
pub const CHANGE_SEPARATOR: char = ',';
/// The id that a new annotation has in the check for a second critical mark (no annotation has it).
pub const NO_ANNOTATION: i64 = 0;

/// How many indexes the alert step needs (`ENSURE_STATE_INDEXES`).
pub const STATE_INDEX_COUNT: i64 = 3;

/// Log event: the extra read connection of a long read could not be opened; the read runs on the
/// shared connection and holds up the other queries.
pub const LOG_READ_CONNECTION_UNAVAILABLE: &str = "read_connection_unavailable";

/// How many snapshots keep their namespace graph and their statistics facts in memory at once.
pub const KEPT_BUILDS: usize = 2;
/// How many walks of usages and impact keep their references in memory at once: the reads of one
/// question to the screen (both directions, the direct usages, the impact).
pub const KEPT_WALKS: usize = 4;

/// The names of the parameters of the statements over `symbol_statistics`.
pub const PARAM_SNAPSHOT: &str = ":snapshot";
/// The group of kinds of a ranking: 1 for types, 0 for callables, NULL for both.
pub const PARAM_IS_TYPE: &str = ":is_type";
/// The language of a ranking, NULL for all.
pub const PARAM_LANGUAGE: &str = ":language";
/// Whether test code counts: 1 if it does.
pub const PARAM_TESTS: &str = ":tests";
/// The wanted assessment of an unused symbol as a number, NULL for all.
pub const PARAM_ASSESSMENT: &str = ":assessment";
/// The languages whose usages are an approximation, each between two unit separators.
pub const PARAM_APPROXIMATE: &str = ":approximate";
/// The first visibility that makes an unused symbol "very likely unused".
pub const PARAM_HIDDEN_FIRST: &str = ":hidden_first";
/// The second visibility that makes an unused symbol "very likely unused".
pub const PARAM_HIDDEN_SECOND: &str = ":hidden_second";
/// The third visibility that makes an unused symbol "very likely unused".
pub const PARAM_HIDDEN_THIRD: &str = ":hidden_third";
/// The most rows of a ranking.
pub const PARAM_LIMIT: &str = ":limit";
/// The ids of the symbols to read, as a JSON list.
pub const PARAM_IDS: &str = ":ids";
/// The number of an assessment in the order of its variants: very likely unused.
pub const ASSESSMENT_NUMBER_LIKELY: i64 = 0;
/// The number of an assessment: public.
pub const ASSESSMENT_NUMBER_PUBLIC: i64 = 1;
/// The number of an assessment: approximate.
pub const ASSESSMENT_NUMBER_APPROXIMATE: i64 = 2;

/// How often the numbers of the statistics are counted on a connection of their own before they
/// are counted under the write lock: a write that comes in meanwhile makes the count useless.
pub const COUNT_ATTEMPTS: u32 = 3;

/// How many measures a statistics row holds: complexity, lines, maintainability, efferent and
/// afferent coupling.
pub const MEASURES: usize = 5;
/// How many yes/no facts tell the groups of the counts apart: used, idle, has a complexity, has
/// a size and has a coupling.
pub const FACET_FLAGS: usize = 5;
/// The columns of `snapshot_statistics`, as the state query returns them.
pub const STATE_CERTAIN_COLUMN: usize = 1;
/// The probable references in the state query.
pub const STATE_PROBABLE_COLUMN: usize = 2;

/// How many read-only connections stay open between the reads that run apart.
pub const IDLE_READERS: usize = 4;

/// A snapshot that holds more than one in this many of the files of the database reads its
/// references in one pass over the table; a smaller one reads them through its files.
pub const WHOLE_TABLE_SHARE: i64 = 2;

/// Log event of one step of writing a snapshot.
pub const LOG_PERSIST_STEP: &str = "persist_step";
/// Step: insert the snapshot header.
pub const STEP_HEADER: &str = "header";
/// Step: drop the indexes and delete expired snapshots before a large full scan is loaded.
pub const STEP_PRUNE_BEFORE: &str = "prune_before_load";
/// Step: insert files, symbols, references, metrics, fingerprints and pending items.
pub const STEP_FACTS: &str = "facts";
/// Step: insert naming rules, parse times, warnings, left-out files and carried-over rows.
pub const STEP_DETAILS: &str = "details";
/// Step: check the integrity of the written facts.
pub const STEP_INTEGRITY: &str = "integrity";
/// Step: build the read models and set the totals.
pub const STEP_READ_MODELS: &str = "read_models";
/// Step: delete the snapshots beyond the retention.
pub const STEP_PRUNE: &str = "prune";
/// Step: commit the transaction of a snapshot write.
pub const STEP_COMMIT: &str = "commit";
/// Step: build the reference indexes after a bulk load.
pub const STEP_REFERENCE_INDEXES: &str = "reference_indexes";
/// Step: insert the references of a snapshot.
pub const STEP_REFERENCES: &str = "references";
/// Step: insert the fingerprints, with the index by hash.
pub const STEP_FINGERPRINTS: &str = "fingerprints";
/// From this many references a scan is a large load, for which the write gets a larger page cache.
pub const LARGE_LOAD_REFERENCES_MIN: usize = 500_000;
/// Page cache of 1 GiB (a negative value is a size in KiB) while a large scan is written. On a
/// test load of 6 million references into a database that held 18 million, 1 GiB and 2 GiB cut
/// the write from 96 s to 56 s and 59 s, so the larger one buys nothing; the cache counts against
/// the memory ceiling of the scan.
pub const LOAD_CACHE_SIZE_KIB: i64 = -1_048_576;
/// Log event: the cache size of the write connection could not be set back.
pub const LOG_CACHE_NOT_RESTORED: &str = "cache_size_not_restored";
