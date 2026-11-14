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
use codeskimr_application::level2::Level2Pass;
use codeskimr_application::metrics::StandardMetrics;
use codeskimr_application::pipeline::{Scanner, ScannerPorts};
use codeskimr_application::ports::{AuthError, Credential, CredentialStore};
use codeskimr_application::ports::{
    FileAnalyzer, FileDiscovery, IndexerSpec, SnippetFingerprinter,
};
use codeskimr_application::resolve::ScopedResolver;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{CredentialId, LanguageId};
use codeskimr_indexer::ProcessIndexRunner;
use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};
use codeskimr_scip::ScipImporter;
use codeskimr_secrets::{KeystoreCredentials, OsKeystore};
use codeskimr_source::{
    DiscoveryConfig, FolderText, GitCache, LanguageMap, SourceFetcher, WalkDiscovery,
};
use codeskimr_storage::SqliteStore;

use crate::clock::SystemClock;
use crate::consts::{BYTES_PER_MB, is_token_credential};
use crate::locations::Locations;
use crate::redact::redact;
pub use crate::shared_keystore::{Secrets, SharedKeystore};

/// What the fetcher of Git sources sees of the secrets: everything except the installation
/// token. A project row that names `installation-token` (written by hand or by an older version)
/// must not make a clone send the token of this installation to a remote.
pub struct ForRemotes(Arc<Secrets>);

impl CredentialStore for ForRemotes {
    fn with_credential<T>(
        &self,
        id: &CredentialId,
        action: impl FnOnce(&Credential) -> Result<T, AuthError>,
    ) -> Result<T, AuthError> {
        if is_token_credential(id.as_str()) {
            return Err(AuthError::NotFound);
        }
        self.0.with_credential(id, action)
    }

    fn store(&self, id: &CredentialId, credential: &Credential) -> Result<(), AuthError> {
        self.0.store(id, credential)
    }

    fn delete(&self, id: &CredentialId) -> Result<(), AuthError> {
        self.0.delete(id)
    }
}

/// The credential store of the operating system.
#[must_use]
pub fn os_secrets() -> Arc<Secrets> {
    Arc::new(KeystoreCredentials::new(SharedKeystore::new(OsKeystore)))
}

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

/// Builds the scanner with every port wired to its adapter; Git logins read from `secrets`.
///
/// # Errors
/// `PRS` failures if the language packs cannot be loaded, `STO` failures of the id lookup.
pub fn build_scanner(
    store: &Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
    events: EventBus,
    secrets: &Arc<Secrets>,
) -> Result<Scanner, CodedError> {
    let Analysis {
        discovery,
        analyzer,
        indexers,
        ..
    } = build_analysis(config, locations)?;
    let ids = Arc::new(IdSequence::starting_at(store.next_free_id()?));
    let pass = Level2Pass {
        runner: Arc::new(ProcessIndexRunner::new(locations.data_dir.clone(), redact)),
        importer: Arc::new(ScipImporter::new(Arc::new(FolderText))),
        consent: store.clone(),
        discovery: Arc::clone(&discovery),
        projects: store.clone(),
        store: store.clone(),
        ids: Arc::clone(&ids),
        settings: config.level2.clone(),
        indexers,
    };
    let ports = ScannerPorts {
        source: Arc::new(SourceFetcher::new(
            GitCache::new(locations.git_cache()),
            Arc::new(ForRemotes(Arc::clone(secrets))),
        )),
        discovery,
        analyzer,
        resolver: Arc::new(ScopedResolver),
        metrics: Arc::new(StandardMetrics),
        projects: store.clone(),
        store: store.clone(),
        clock: Arc::new(SystemClock),
    };
    Ok(Scanner::new(ports, &ids, events).with_level2(pass))
}

/// The two ports that read code: walking the files and analysing one of them. The scan and the
/// convention check use the same pair, so both see the same files.
pub struct Analysis {
    /// Walks the folder.
    pub discovery: Arc<dyn FileDiscovery>,
    /// Parses one file; fingerprints are on.
    pub analyzer: Arc<dyn FileAnalyzer>,
    /// Fingerprints a snippet with the same parameters as the analyzer.
    pub snippets: Arc<dyn SnippetFingerprinter>,
    /// How level 2 runs the indexer of each language whose pack names one.
    pub indexers: Vec<(LanguageId, IndexerSpec)>,
}

/// Builds the discovery and the analyzer from the installed language packs.
///
/// # Errors
/// `PRS` failures if the language packs cannot be loaded.
pub fn build_analysis(config: &Config, locations: &Locations) -> Result<Analysis, CodedError> {
    let packs = LanguagePacks::load(&locations.languages_dir).map_err(CodedError::caused_by)?;
    let languages: LanguageMap = packs
        .iter()
        .flat_map(|pack| {
            pack.extensions()
                .iter()
                .map(|extension| (extension.as_str(), pack.id().clone()))
        })
        .collect();
    let indexers = packs
        .iter()
        .filter_map(|summary| Some((summary.id().clone(), summary.indexer()?.clone())))
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
    let analyzer = Arc::new(analyzer);
    Ok(Analysis {
        discovery: Arc::new(discovery),
        snippets: analyzer.clone(),
        analyzer,
        indexers,
    })
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;

    use super::*;
    use crate::memory_keystore::secrets;

    fn credential() -> Credential {
        Credential::new(SecretString::from("user"), SecretString::from("secret"))
    }

    #[test]
    fn the_fetcher_never_gets_the_installation_token() {
        use crate::consts::TOKEN_CREDENTIAL_ID;
        let secrets = secrets();
        let token = CredentialId::new(TOKEN_CREDENTIAL_ID).unwrap();
        let work = CredentialId::new("work").unwrap();
        secrets.store(&token, &credential()).unwrap();
        secrets.store(&work, &credential()).unwrap();
        let remotes = ForRemotes(Arc::clone(&secrets));
        // The token exists, the fetcher still cannot see it; any other credential is lent.
        assert!(secrets.with_credential(&token, |_| Ok(())).is_ok());
        assert!(matches!(
            remotes.with_credential(&token, |_| Ok(())),
            Err(AuthError::NotFound)
        ));
        assert!(remotes.with_credential(&work, |_| Ok(())).is_ok());
    }
}
