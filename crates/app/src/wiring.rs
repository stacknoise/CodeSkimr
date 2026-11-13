//! The wiring: creates the adapters and hands them to the application layer as ports.
//!
//! This is the only place that knows concrete adapter types (blueprint 4.1).

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ids::IdSequence;
use codeskimr_application::metrics::StandardMetrics;
use codeskimr_application::pipeline::{Scanner, ScannerPorts};
use codeskimr_application::resolve::ScopedResolver;
use codeskimr_codes::ErrorCode;
use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};
use codeskimr_secrets::{OsCredentialStore, OsKeystore};
use codeskimr_source::{DiscoveryConfig, GitCache, LanguageMap, SourceFetcher, WalkDiscovery};
use codeskimr_storage::SqliteStore;

use crate::clock::SystemClock;
use crate::consts::BYTES_PER_MB;
use crate::locations::Locations;

/// Opens (and creates, if needed) the database of the data folder.
///
/// # Errors
/// `STO` failures of the database, `APP-901` if the retention is zero (the configuration
/// forbids it).
pub fn open_store(locations: &Locations, config: &Config) -> Result<Arc<SqliteStore>, CodedError> {
    let retention = NonZeroU32::new(config.snapshots.retention_count)
        .ok_or_else(|| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    SqliteStore::open(&locations.database(), retention).map(Arc::new)
}

/// Builds the scanner with every port wired to its adapter.
///
/// # Errors
/// `PRS` failures if the language packs cannot be loaded, `STO` failures of the id lookup.
pub fn build_scanner(
    store: &Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
    events: EventBus,
) -> Result<Scanner, CodedError> {
    let packs = LanguagePacks::load(&locations.languages_dir).map_err(CodedError::caused_by)?;
    let languages: LanguageMap = packs
        .iter()
        .flat_map(|pack| {
            pack.extensions()
                .iter()
                .map(|extension| (extension.as_str(), pack.id().clone()))
        })
        .collect();
    let timeout = Duration::from_millis(u64::from(config.scan.parse_timeout_ms));
    let analyzer = TreeSitterAnalyzer::new(packs, timeout)
        .with_fingerprints(&config.similarity)
        .map_err(CodedError::caused_by)?;
    let discovery = WalkDiscovery::new(
        DiscoveryConfig {
            excluded_folders: config.scan.excluded_folders.clone(),
            max_total_bytes: u64::from(config.scan.max_source_mb).saturating_mul(BYTES_PER_MB),
        },
        languages,
    );
    let ports = ScannerPorts {
        source: Arc::new(SourceFetcher::new(
            GitCache::new(locations.git_cache()),
            Arc::new(OsCredentialStore::new(OsKeystore)),
        )),
        discovery: Arc::new(discovery),
        analyzer: Arc::new(analyzer),
        resolver: Arc::new(ScopedResolver),
        metrics: Arc::new(StandardMetrics),
        projects: store.clone(),
        store: store.clone(),
        clock: Arc::new(SystemClock),
    };
    let ids = Arc::new(IdSequence::starting_at(store.next_free_id()?));
    Ok(Scanner::new(ports, &ids, events))
}
