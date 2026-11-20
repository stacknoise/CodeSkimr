//! The second pass of a scan: runs the indexer of each language, if the user allowed it, and
//! replaces the level 1 references of the files it checked where it answers (ADR 0033).
//!
//! The pass never fails the scan. Whatever goes wrong, the snapshot of level 1 stays as it was
//! written and the report says why a language was skipped.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    FileId, FolderPath, LanguageId, Level2Stage, Position, Principal, Project, Reference,
    RelativePath, Snapshot, SourceFile, Symbol,
};

use super::authorize_run;
use super::renumber::renumbered;
use super::report::{LanguageReport, Level2Outcome, Level2Result, Skipped};
use crate::cancel::CancelToken;
use crate::config::Level2;
use crate::error::CodedError;
use crate::events::{Event, EventBus, Level2Progressed};
use crate::ids::IdSequence;
use crate::ports::{
    ConsentStore, FileDiscovery, ImportRequest, Imported, IndexJob, IndexRunner, IndexerSpec,
    IndexerStatus, Level2Replacement, PreviousIndex, ProjectRepository, ReferenceImporter,
    SnapshotStore, StoredNames,
};

/// The ports and settings of the second pass, wired by the composition root.
pub struct Level2Pass {
    /// Starts the indexers.
    pub runner: Arc<dyn IndexRunner>,
    /// Reads an index and maps it.
    pub importer: Arc<dyn ReferenceImporter>,
    /// The consent of the user.
    pub consent: Arc<dyn ConsentStore>,
    /// Hashes the files again after the run.
    pub discovery: Arc<dyn FileDiscovery>,
    /// Reads the snapshot that was just written.
    pub projects: Arc<dyn ProjectRepository>,
    /// Writes the new references.
    pub store: Arc<dyn SnapshotStore>,
    /// Ids of the new references.
    pub ids: Arc<IdSequence>,
    /// Largest source file whose text the importer may read, in bytes (the scan's own limit).
    pub max_file_bytes: u64,
    /// The settings of the user (`level2.*`).
    pub settings: Level2,
    /// How to run the indexer of each language that has one. Languages that name the same
    /// indexer (TypeScript, JavaScript and TSX; Java and Kotlin) share one run.
    pub indexers: Vec<(LanguageId, IndexerSpec)>,
    /// Where the run tells which language it works on (ADR 0072).
    pub events: EventBus,
}

/// The languages that one run of one indexer covers.
struct Group<'a> {
    spec: &'a IndexerSpec,
    languages: Vec<&'a LanguageId>,
}

/// What every group of one scan has in common.
struct Scan<'a> {
    principal: &'a Principal,
    snapshot: &'a Snapshot,
    project: Project,
    root: &'a FolderPath,
    cancel: &'a CancelToken,
    files: Vec<SourceFile>,
    index: PreviousIndex,
}

impl Level2Pass {
    /// Runs the indexers for the languages of `snapshot`, whose files lie in `root`, and reports
    /// per language. After a cancel the run in progress is killed and the rest is skipped.
    #[must_use]
    pub fn run(
        &self,
        principal: &Principal,
        snapshot: &Snapshot,
        root: &FolderPath,
        cancel: &CancelToken,
    ) -> Level2Result {
        let scan = match self.load(principal, snapshot, root, cancel) {
            Ok(scan) => scan,
            Err(cause) => {
                return Level2Result {
                    reports: self.skip_all(&Skipped::Failed(cause.code())),
                    snapshot: snapshot.clone(),
                };
            }
        };
        let present: HashSet<&LanguageId> = scan.files.iter().map(|file| &file.language).collect();
        let mut result = Level2Result {
            reports: Vec::new(),
            snapshot: snapshot.clone(),
        };
        for group in self.groups(&present) {
            let outcome = self.outcome(&scan, &group);
            if let (Level2Outcome::Checked { .. }, Some(fresh)) = (&outcome.0, outcome.1) {
                result.snapshot = fresh;
            }
            result
                .reports
                .extend(group.languages.iter().map(|language| LanguageReport {
                    language: (*language).clone(),
                    outcome: outcome.0.clone(),
                }));
        }
        result
    }

    fn load<'a>(
        &self,
        principal: &'a Principal,
        snapshot: &'a Snapshot,
        root: &'a FolderPath,
        cancel: &'a CancelToken,
    ) -> Result<Scan<'a>, CodedError> {
        Ok(Scan {
            principal,
            snapshot,
            project: self.projects.project(snapshot.project)?,
            root,
            cancel,
            files: self.projects.latest_files(snapshot.project)?,
            index: self.projects.latest_index(snapshot.project)?,
        })
    }

    /// Whether `principal` may run the indexers of `project` now (`authorize_run`), without
    /// running anything. The run asks again when it starts, so a consent that goes away in
    /// between is honoured.
    ///
    /// # Errors
    /// `NotPermitted` (`SEC-403`) or `NotConfirmed` (`SEC-301`).
    pub fn authorize(
        &self,
        principal: &Principal,
        project: &Project,
        root: &FolderPath,
    ) -> Result<(), Skipped> {
        authorize_run(principal, &self.settings, &self.consent, project, root)
            .map(|_| ())
            .map_err(Skipped::from)
    }

    /// The languages that have an indexer.
    pub fn languages(&self) -> impl Iterator<Item = &LanguageId> {
        self.indexers.iter().map(|(language, _)| language)
    }

    fn announce(&self, snapshot: &Snapshot, language: &LanguageId, stage: Level2Stage) {
        self.events
            .publish(&Event::Level2Progressed(Level2Progressed {
                project: snapshot.project,
                language: language.clone(),
                stage,
            }));
    }

    fn skip_all(&self, reason: &Skipped) -> Vec<LanguageReport> {
        self.indexers
            .iter()
            .map(|(language, _)| LanguageReport {
                language: language.clone(),
                outcome: Level2Outcome::Skipped(reason.clone()),
            })
            .collect()
    }

    /// The indexers grouped by name, for the languages that have files.
    fn groups(&self, present: &HashSet<&LanguageId>) -> Vec<Group<'_>> {
        let mut by_name: BTreeMap<&str, Group<'_>> = BTreeMap::new();
        for (language, spec) in &self.indexers {
            if present.contains(language) {
                by_name
                    .entry(spec.name.as_str())
                    .or_insert_with(|| Group {
                        spec,
                        languages: Vec::new(),
                    })
                    .languages
                    .push(language);
            }
        }
        by_name.into_values().collect()
    }

    fn outcome(&self, scan: &Scan<'_>, group: &Group<'_>) -> (Level2Outcome, Option<Snapshot>) {
        if scan.cancel.is_cancelled() {
            return (Level2Outcome::Skipped(Skipped::Cancelled), None);
        }
        match self.try_group(scan, group) {
            Ok((outcome, snapshot)) => (outcome, Some(snapshot)),
            Err(skipped) => (Level2Outcome::Skipped(skipped), None),
        }
    }

    fn try_group(
        &self,
        scan: &Scan<'_>,
        group: &Group<'_>,
    ) -> Result<(Level2Outcome, Snapshot), Skipped> {
        let permit = authorize_run(
            scan.principal,
            &self.settings,
            &self.consent,
            &scan.project,
            scan.root,
        )
        .map_err(Skipped::from)?;
        if let IndexerStatus::Unavailable { problems, .. } = self.runner.check(group.spec) {
            return Err(Skipped::Unavailable(problems));
        }
        let language = group.languages.first().copied().ok_or(Skipped::Stale)?;
        let job = IndexJob {
            language,
            spec: group.spec,
            limits: self.settings.limits(),
            permit: &permit,
        };
        self.announce(scan.snapshot, language, Level2Stage::Running);
        let artifact = self.runner.run(&job, scan.cancel).map_err(Skipped::from)?;
        let (current, stale) = self.unchanged(scan, group)?;
        self.announce(scan.snapshot, language, Level2Stage::Importing);
        let imported = self.import(scan, group.spec, artifact.path(), &current)?;
        if scan.cancel.is_cancelled() {
            return Err(Skipped::Cancelled);
        }
        self.replace(scan.snapshot, &imported, stale)
    }

    /// Reads the index at `index` against the `files` and the symbols of the scan.
    fn import(
        &self,
        scan: &Scan<'_>,
        spec: &IndexerSpec,
        index: &Path,
        files: &[SourceFile],
    ) -> Result<Imported, Skipped> {
        let import_sites = self.import_sites(scan, files);
        self.importer
            .import(&ImportRequest {
                index,
                root: scan.root,
                files,
                symbols: &scan.index.symbols,
                references: &scan.index.references,
                max_index_bytes: self.settings.max_index_bytes,
                max_file_bytes: self.max_file_bytes,
                min_mapped: self.settings.min_mapped,
                symbol_kinds: &spec.symbol_kinds,
                separator: &spec.separator,
                library_root: &spec.library_root,
                max_created_symbols: usize::try_from(self.settings.max_created_symbols)
                    .unwrap_or(usize::MAX),
                import_sites: &import_sites,
            })
            .map_err(Skipped::from)
    }

    /// Where level 1 read the names of the imports of `files`. A snapshot that does not hold its
    /// names gives none: its imports then count as uses, as before.
    fn import_sites(&self, scan: &Scan<'_>, files: &[SourceFile]) -> Vec<(FileId, Position)> {
        let ids: Vec<FileId> = files.iter().map(|file| file.id).collect();
        self.projects
            .stored_names(scan.snapshot.id, StoredNames::ImportsOf(&ids))
            .unwrap_or_default()
            .into_iter()
            .map(|name| (name.file, name.span.start()))
            .collect()
    }

    /// The files of the group whose content is still the one of the snapshot, and the number of
    /// those that changed. The hash is checked after the run, which may have taken minutes.
    fn unchanged(
        &self,
        scan: &Scan<'_>,
        group: &Group<'_>,
    ) -> Result<(Vec<SourceFile>, u64), Skipped> {
        let wanted = |file: &&SourceFile| group.languages.contains(&&file.language);
        let group_files: Vec<SourceFile> = scan.files.iter().filter(wanted).cloned().collect();
        // Only the files of this group are read again, not the whole project (issue #231).
        let unchanged = self
            .discovery
            .unchanged_files(scan.root, &group_files, scan.cancel)
            .map_err(Skipped::from)?;
        let current: HashSet<&RelativePath> = unchanged.iter().collect();
        let all = group_files.len();
        let kept: Vec<SourceFile> = group_files
            .iter()
            .filter(|file| current.contains(&file.path))
            .cloned()
            .collect();
        let stale = u64::try_from(all.saturating_sub(kept.len())).unwrap_or(u64::MAX);
        if kept.is_empty() {
            Err(Skipped::Stale)
        } else {
            Ok((kept, stale))
        }
    }

    fn replace(
        &self,
        snapshot: &Snapshot,
        imported: &Imported,
        stale: u64,
    ) -> Result<(Level2Outcome, Snapshot), Skipped> {
        let (symbols, references) = renumber(imported, &self.ids);
        let fresh = self
            .store
            .replace_level2(Level2Replacement {
                snapshot: snapshot.id,
                files: imported.files.clone(),
                symbols,
                references,
            })
            .map_err(Skipped::from)?;
        let checked = Level2Outcome::Checked {
            files: u64::try_from(imported.files.len()).unwrap_or(u64::MAX),
            stale,
            mapped: imported.mapped,
            dropped: imported.dropped,
            outside_scan: imported.outside_scan,
        };
        Ok((checked, fresh))
    }
}

/// The created symbols and the references of `imported` with ids reserved from the shared
/// sequence. The importer numbers the symbols consecutively from the first free id of the
/// snapshot; every id from there on, as a symbol, parent or target, moves by the same distance.
fn renumber(imported: &Imported, ids: &IdSequence) -> (Vec<Symbol>, Vec<Reference>) {
    let count = |len: usize| u64::try_from(len).unwrap_or(u64::MAX);
    let first_reference = ids.reserve(count(imported.references.len()));
    let first_symbol = ids.reserve(count(imported.symbols.len()));
    renumbered(
        &imported.symbols,
        imported.references.iter(),
        first_symbol,
        first_reference,
    )
}

impl From<CodedError> for Skipped {
    fn from(cause: CodedError) -> Self {
        match cause.code() {
            ErrorCode::IndexerNotPermitted => Self::NotPermitted,
            ErrorCode::IndexerNotConfirmed => Self::NotConfirmed,
            ErrorCode::ScanCancelled => Self::Cancelled,
            other => Self::Failed(other),
        }
    }
}
