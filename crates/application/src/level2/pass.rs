//! The second pass of a scan: runs the indexer of each language, if the user allowed it, and
//! replaces the level 1 references of the files it checked (ADR 0033).
//!
//! The pass never fails the scan. Whatever goes wrong, the snapshot of level 1 stays as it was
//! written and the report says why a language was skipped.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    FolderPath, LanguageId, Principal, ReferenceId, RelativePath, Snapshot, SourceFile,
};

use super::authorize_run;
use super::report::{LanguageReport, Level2Outcome, Level2Result, Skipped};
use crate::cancel::CancelToken;
use crate::config::Level2;
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{
    ConsentStore, FileDiscovery, ImportRequest, Imported, IndexJob, IndexRunner, IndexerSpec,
    IndexerStatus, PreviousIndex, ProjectRepository, ReferenceImporter, ReferenceReplacement,
    SnapshotStore,
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
            root,
            cancel,
            files: self.projects.latest_files(snapshot.project)?,
            index: self.projects.latest_index(snapshot.project)?,
        })
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
            scan.snapshot.project,
            Path::new(scan.root.as_str()),
        )
        .map_err(Skipped::from)?;
        if let IndexerStatus::Unavailable(problems) = self.runner.check(group.spec) {
            return Err(Skipped::Unavailable(problems));
        }
        let language = group.languages.first().copied().ok_or(Skipped::Stale)?;
        let job = IndexJob {
            language,
            spec: group.spec,
            limits: self.settings.limits(),
            permit: &permit,
        };
        let artifact = self.runner.run(&job, scan.cancel).map_err(Skipped::from)?;
        let (current, stale) = self.unchanged(scan, group)?;
        let imported = self
            .importer
            .import(&ImportRequest {
                index: artifact.path(),
                root: scan.root,
                files: &current,
                symbols: &scan.index.symbols,
                references: &scan.index.references,
                max_index_bytes: self.settings.max_index_bytes,
                max_file_bytes: self.max_file_bytes,
                min_mapped: self.settings.min_mapped,
            })
            .map_err(Skipped::from)?;
        self.replace(scan.snapshot, &imported, stale)
    }

    /// The files of the group whose content is still the one of the snapshot, and the number of
    /// those that changed. The hash is checked after the run, which may have taken minutes.
    fn unchanged(
        &self,
        scan: &Scan<'_>,
        group: &Group<'_>,
    ) -> Result<(Vec<SourceFile>, u64), Skipped> {
        let discovered = self
            .discovery
            .discover(scan.root, &scan.files, scan.cancel)
            .map_err(Skipped::from)?;
        let current: HashSet<&RelativePath> = discovered
            .iter()
            .filter(|file| !file.changed)
            .map(|file| &file.path)
            .collect();
        let wanted = |file: &&SourceFile| group.languages.contains(&&file.language);
        let all = scan.files.iter().filter(wanted).count();
        let kept: Vec<SourceFile> = scan
            .files
            .iter()
            .filter(wanted)
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
        let count = u64::try_from(imported.references.len()).unwrap_or(u64::MAX);
        let first = self.ids.reserve(count);
        let references = imported
            .references
            .iter()
            .zip(first..)
            .map(|(reference, id)| codeskimr_domain::Reference {
                id: ReferenceId::new(id),
                ..*reference
            })
            .collect();
        let fresh = self
            .store
            .replace_references(ReferenceReplacement {
                snapshot: snapshot.id,
                files: imported.files.clone(),
                references,
            })
            .map_err(Skipped::from)?;
        let checked = Level2Outcome::Checked {
            files: u64::try_from(imported.files.len()).unwrap_or(u64::MAX),
            stale,
            mapped: imported.mapped,
            dropped: imported.dropped,
        };
        Ok((checked, fresh))
    }
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
