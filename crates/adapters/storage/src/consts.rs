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
/// Whether an edge of the layer graph violates the layering; not computed yet (see docs).
pub const NOT_VIOLATING: bool = false;
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
/// Containers that appear in the explorer tree next to the types of `TYPE_KINDS`.
pub const CONTAINER_KINDS: &[codeskimr_domain::SymbolKind] = &[
    codeskimr_domain::SymbolKind::Namespace,
    codeskimr_domain::SymbolKind::Module,
];
