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
    FileId, FolderPath, LanguageId, Level2Stage, Principal, Project, ProjectId, Reference,
    RelativePath, Snapshot, SourceFile, Span, Symbol,
};

use super::authorize_run;
use super::renumber::renumbered;
use super::report::{LanguageReport, Level2Outcome, Level2Result, Skipped};
use super::tally::Tally;
use crate::cancel::CancelToken;
use crate::config::Level2;
use crate::consts::WORKSPACE_SEPARATOR;
use crate::error::CodedError;
use crate::events::{Event, EventBus, Level2Progressed};
use crate::ids::{IdSequence, ProjectIds, project_of_snapshot};
use crate::ports::{
    ConsentStore, FileDiscovery, ImportRequest, Imported, IndexJob, IndexRunner, IndexerSpec,
    IndexerStatus, Level2Replacement, PreviousIndex, ProjectRepository, ReferenceImporter,
    RunPermit, SnapshotStore, StoredNames,
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
    /// Ids of the new references, one sequence for each project.
    pub ids: Arc<ProjectIds>,
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
        self.run_only(principal, (snapshot, root), cancel, None)
    }

    /// Like [`Level2Pass::run`], but with `only` set it covers that language alone: the others
    /// are neither run nor reported.
    #[must_use]
    pub fn run_only(
        &self,
        principal: &Principal,
        (snapshot, root): (&Snapshot, &FolderPath),
        cancel: &CancelToken,
        only: Option<&LanguageId>,
    ) -> Level2Result {
        let scan = match self.load(principal, snapshot, root, cancel) {
            Ok(scan) => scan,
            Err(cause) => {
                return Level2Result {
                    reports: self.skip_all(snapshot.project, only, &Skipped::Failed(cause.code())),
                    snapshot: snapshot.clone(),
                };
            }
        };
        let present: HashSet<&LanguageId> = scan
            .files
            .iter()
            .map(|file| &file.language)
            .filter(|language| only.is_none_or(|one| one == *language))
            .collect();
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

    /// The languages that have an indexer and files in the latest snapshot of `project`. If the
    /// files cannot be read, all languages with an indexer, so that no skip goes unreported.
    #[must_use]
    pub fn languages_in(&self, project: ProjectId, only: Option<&LanguageId>) -> Vec<&LanguageId> {
        let present: Option<HashSet<LanguageId>> = self
            .projects
            .latest_files(project)
            .ok()
            .map(|files| files.into_iter().map(|file| file.language).collect());
        self.languages()
            .filter(|language| only.is_none_or(|one| one == *language))
            .filter(|language| present.as_ref().is_none_or(|set| set.contains(*language)))
            .collect()
    }

    fn skip_all(
        &self,
        project: ProjectId,
        only: Option<&LanguageId>,
        reason: &Skipped,
    ) -> Vec<LanguageReport> {
        self.languages_in(project, only)
            .into_iter()
            .map(|language| LanguageReport {
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
        let files: Vec<SourceFile> = scan
            .files
            .iter()
            .filter(|file| group.languages.contains(&&file.language))
            .cloned()
            .collect();
        let paths: Vec<&RelativePath> = files.iter().map(|file| &file.path).collect();
        let workspaces = self
            .runner
            .workspaces(group.spec, permit.project_root(), &paths);
        if workspaces.is_empty() {
            return Err(Skipped::NoProjectFile);
        }
        if let IndexerStatus::Unavailable { problems, .. } = self.runner.check(group.spec) {
            return Err(Skipped::Unavailable(problems));
        }
        let mut tally = Tally::default();
        for workspace in &workspaces {
            let mine = in_workspace(&files, workspace.as_ref());
            match self.run_workspace(scan, group, (&permit, workspace.as_ref()), &mine) {
                Ok((outcome, fresh)) => tally.checked(&outcome, fresh),
                Err(Skipped::Cancelled) => {
                    tally.cancel();
                    break;
                }
                Err(reason) => tally.failed(reason, mine.len()),
            }
        }
        tally.outcome()
    }

    /// One run of the indexer in `workspace` (`None`: the project root) over the `files` below it,
    /// and the import of its index.
    fn run_workspace(
        &self,
        scan: &Scan<'_>,
        group: &Group<'_>,
        (permit, workspace): (&RunPermit, Option<&RelativePath>),
        files: &[SourceFile],
    ) -> Result<(Level2Outcome, Snapshot), Skipped> {
        if scan.cancel.is_cancelled() {
            return Err(Skipped::Cancelled);
        }
        let language = group.languages.first().copied().ok_or(Skipped::Stale)?;
        let job = IndexJob {
            language,
            spec: group.spec,
            limits: self.settings.limits_of(&group.spec.name),
            workspace,
            permit,
        };
        self.announce(scan.snapshot, language, Level2Stage::Running);
        let artifact = self.runner.run(&job, scan.cancel).map_err(Skipped::from)?;
        let (current, stale) = self.unchanged(scan, files)?;
        self.announce(scan.snapshot, language, Level2Stage::Importing);
        let imported = self.import(scan, (group.spec, workspace), artifact.path(), &current)?;
        if scan.cancel.is_cancelled() {
            return Err(Skipped::Cancelled);
        }
        self.replace(scan.snapshot, &imported, stale)
    }

    /// Reads the index at `index` against the `files` and the symbols of the scan.
    fn import(
        &self,
        scan: &Scan<'_>,
        (spec, workspace): (&IndexerSpec, Option<&RelativePath>),
        index: &Path,
        files: &[SourceFile],
    ) -> Result<Imported, Skipped> {
        let import_sites = self.import_sites(scan, files)?;
        self.importer
            .import(&ImportRequest {
                index,
                workspace,
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
    /// names gives none (its imports then count as uses, as before); a failure of the store
    /// fails the import.
    fn import_sites(
        &self,
        scan: &Scan<'_>,
        files: &[SourceFile],
    ) -> Result<Vec<(FileId, Span)>, Skipped> {
        let ids: Vec<FileId> = files.iter().map(|file| file.id).collect();
        let names = self
            .projects
            .stored_names(scan.snapshot.id, StoredNames::ImportsOf(&ids))
            .map_err(Skipped::from)?;
        Ok(names
            .into_iter()
            .map(|name| (name.file, name.span))
            .collect())
    }

    /// The `files` whose content is still the one of the snapshot, and the number of those that
    /// changed. The hash is checked after the run, which may have taken minutes.
    fn unchanged(
        &self,
        scan: &Scan<'_>,
        group_files: &[SourceFile],
    ) -> Result<(Vec<SourceFile>, u64), Skipped> {
        // Only the files of this run are read again, not the whole project (issue #231).
        let unchanged = self
            .discovery
            .unchanged_files(scan.root, group_files, scan.cancel)
            .map_err(Skipped::from)?;
        let current: HashSet<&RelativePath> = unchanged.iter().collect();
        let kept: Vec<SourceFile> = group_files
            .iter()
            .filter(|file| current.contains(&file.path))
            .cloned()
            .collect();
        let stale = u64::try_from(group_files.len().saturating_sub(kept.len())).unwrap_or(u64::MAX);
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
        let project = project_of_snapshot(snapshot.id);
        let first_free = self
            .store
            .next_free_ids(project)
            .map_err(|cause| Skipped::Failed(cause.code()))?;
        let (symbols, references) = renumber(imported, &self.ids.of(project, first_free));
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

/// The `files` that lie below `workspace` (`None`: the project root, all of them).
fn in_workspace(files: &[SourceFile], workspace: Option<&RelativePath>) -> Vec<SourceFile> {
    let Some(folder) = workspace else {
        return files.to_vec();
    };
    let prefix = [folder.as_str(), ""].join(WORKSPACE_SEPARATOR);
    files
        .iter()
        .filter(|file| file.path.as_str().starts_with(&prefix))
        .cloned()
        .collect()
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
