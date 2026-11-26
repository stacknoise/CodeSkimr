//! The ports on the set of project files: every call goes to the file of the project it names
//! (ADR 0095).

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ids::{project_of_raw, project_of_snapshot};
use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnnotationCreation, AnnotationFilter,
    AnnotationStore, AnnotationUpdate, CodeIndex, CodeOutline, CodeSpan, CodeStatistics,
    Confirmation, Confirmed, ConsentGrant, ConsentRecord, ConsentStore, DependencyPage,
    DependencyQuery, FileHit, ImpactPage, LanguageShare, Level2Replacement, MemberRow, OutlineNode,
    PendingReference, PreviousIndex, PreviousState, ProjectCatalog, ProjectOverview,
    ProjectRepository, RankedRows, RankingRequest, ReferenceSite, ReferenceSites, ReportWrite,
    SameName, ScanEntry, ScanReport, ScanReportStore, ScopeStore, SearchHit, SimilarHit,
    SimilarQuery, SimilarityIndex, SnapshotData, SnapshotStore, StatisticsSetup, StatisticsSummary,
    StoredNames, StoredReferences, SymbolDetail, SymbolKey, SymbolState, SymbolStates,
    ToolReportStore, TypeFocus, UnusedCounts, UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Alert, AlertId, Annotation, AnnotationId, ContentHash, FileId, Fingerprints, LayerEdge,
    NamespaceGraph, Project, ProjectId, QualifiedName, RankingQuery, Reference, RelativePath,
    Scope, Snapshot, SnapshotId, SourceFile, Symbol, SymbolBaseline, SymbolFacts, SymbolId,
    SymbolName, SymbolPair, ToolReport, UsageQuery,
};

use super::SqliteStore;
use super::consts::LOG_PROJECT_SKIPPED;
use crate::error::StorageError;
use crate::store::ProjectDb;

impl SqliteStore {
    /// The file of a project.
    fn of_project(&self, project: ProjectId) -> Result<Arc<ProjectDb>, CodedError> {
        self.db(project).map_err(CodedError::from)
    }

    /// The file that holds a snapshot.
    fn of_snapshot(&self, snapshot: SnapshotId) -> Result<Arc<ProjectDb>, CodedError> {
        self.of_project(project_of_snapshot(snapshot))
    }

    /// The file that holds an annotation.
    fn of_annotation(&self, annotation: AnnotationId) -> Result<Arc<ProjectDb>, CodedError> {
        self.of_project(project_of_raw(annotation.get()))
    }

    /// The file that holds an alert.
    fn of_alert(&self, alert: AlertId) -> Result<Arc<ProjectDb>, CodedError> {
        self.of_project(project_of_raw(alert.get()))
    }
}

/// Implements methods of a port that only hand the call on to the file their first argument
/// names: `$file` finds the file from the argument.
macro_rules! forward {
    ($port:ident via $file:ident: $(fn $name:ident(&self, $key:ident: $kty:ty $(, $arg:ident: $aty:ty)* $(,)?) -> $ret:ty;)+) => {
        $(
            fn $name(&self, $key: $kty $(, $arg: $aty)*) -> $ret {
                $port::$name(&*self.$file($key)?, $key $(, $arg)*)
            }
        )+
    };
}

impl ProjectRepository for SqliteStore {
    forward! { ProjectRepository via of_project:
        fn project(&self, id: ProjectId) -> Result<Project, CodedError>;
        fn scope(&self, id: ProjectId) -> Result<Scope, CodedError>;
        fn latest_files(&self, id: ProjectId) -> Result<Vec<SourceFile>, CodedError>;
        fn latest_index(&self, id: ProjectId) -> Result<PreviousIndex, CodedError>;
        fn latest_state(&self, id: ProjectId) -> Result<PreviousState, CodedError>;
        fn latest_head(&self, id: ProjectId) -> Result<PreviousState, CodedError>;
        fn latest_naming_rules(&self, id: ProjectId) -> Result<Option<String>, CodedError>;
    }

    fn project_for_removal(&self, id: ProjectId) -> Result<Project, CodedError> {
        let db = self.db_any(id).map_err(CodedError::from)?;
        ProjectRepository::project_for_removal(&*db, id)
    }

    forward! { ProjectRepository via of_snapshot:
        fn snapshot_symbols(&self, snapshot: SnapshotId) -> Result<Vec<Symbol>, CodedError>;
        fn stored_references(&self, snapshot: SnapshotId, which: StoredReferences<'_>) -> Result<Vec<Reference>, CodedError>;
        fn verified_files(&self, snapshot: SnapshotId) -> Result<Vec<FileId>, CodedError>;
        fn reference_pairs(&self, snapshot: SnapshotId, files: &[FileId], without: &[SymbolId]) -> Result<Vec<SymbolPair>, CodedError>;
        fn stored_names(&self, snapshot: SnapshotId, which: StoredNames<'_>) -> Result<Vec<PendingReference>, CodedError>;
    }
}

impl SnapshotStore for SqliteStore {
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError> {
        let db = self.of_project(data.snapshot.project)?;
        SnapshotStore::persist(&*db, data)
    }

    fn replace_level2(&self, replacement: Level2Replacement) -> Result<Snapshot, CodedError> {
        let db = self.of_snapshot(replacement.snapshot)?;
        SnapshotStore::replace_level2(&*db, replacement)
    }

    fn confirm(&self, scan: &Confirmation) -> Result<Option<Confirmed>, CodedError> {
        SnapshotStore::confirm(&*self.of_project(scan.project)?, scan)
    }

    fn next_free_ids(&self, project: ProjectId) -> Result<(u64, u64), CodedError> {
        SnapshotStore::next_free_ids(&*self.of_project(project)?, project)
    }
}

impl CodeIndex for SqliteStore {
    forward! { CodeIndex via of_project:
        fn latest_snapshot(&self, project: ProjectId) -> Result<Option<Snapshot>, CodedError>;
        fn scan_history(&self, project: ProjectId) -> Result<Vec<ScanEntry>, CodedError>;
    }

    forward! { CodeIndex via of_snapshot:
        fn search(&self, snapshot: SnapshotId, query: &str, limit: u32) -> Result<Vec<SearchHit>, CodedError>;
        fn symbol(&self, snapshot: SnapshotId, key: &SymbolKey) -> Result<Option<SymbolDetail>, CodedError>;
        fn symbol_card(&self, snapshot: SnapshotId, id: SymbolId) -> Result<Option<SymbolDetail>, CodedError>;
        fn search_files(&self, snapshot: SnapshotId, query: &str, limit: u32) -> Result<Vec<FileHit>, CodedError>;
        fn code_spans(&self, snapshot: SnapshotId, path: &RelativePath, lines: (u32, u32)) -> Result<Vec<CodeSpan>, CodedError>;
        fn layer_edges(&self, snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError>;
        fn namespace_graph(&self, snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError>;
        fn type_focus(&self, snapshot: SnapshotId, symbol: SymbolId) -> Result<Option<TypeFocus>, CodedError>;
    }
}

impl CodeOutline for SqliteStore {
    forward! { CodeOutline via of_snapshot:
        fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError>;
        fn member_rows(&self, snapshot: SnapshotId, symbol: SymbolId) -> Result<Vec<MemberRow>, CodedError>;
    }
}

impl UsageIndex for SqliteStore {
    forward! { UsageIndex via of_snapshot:
        fn usages(&self, snapshot: SnapshotId, query: &UsageQuery, limit: u32) -> Result<UsagePage, CodedError>;
        fn impact(&self, snapshot: SnapshotId, symbol: SymbolId, depth: u32, limit: u32) -> Result<ImpactPage, CodedError>;
        fn dependencies(&self, snapshot: SnapshotId, query: &DependencyQuery, limit: u32) -> Result<DependencyPage, CodedError>;
    }
}

impl SimilarityIndex for SqliteStore {
    forward! { SimilarityIndex via of_snapshot:
        fn fingerprints_of(&self, snapshot: SnapshotId, symbol: SymbolId) -> Result<Option<Fingerprints>, CodedError>;
        fn similar(&self, snapshot: SnapshotId, query: &SimilarQuery<'_>) -> Result<Vec<SimilarHit>, CodedError>;
    }
}

impl ReferenceSites for SqliteStore {
    forward! { ReferenceSites via of_snapshot:
        fn references_from(&self, snapshot: SnapshotId, files: &[RelativePath]) -> Result<Vec<ReferenceSite>, CodedError>;
    }
}

impl SymbolStates for SqliteStore {
    forward! { SymbolStates via of_snapshot:
        fn states(&self, snapshot: SnapshotId, names: &[QualifiedName]) -> Result<Vec<SymbolState>, CodedError>;
        fn child_hashes(&self, snapshot: SnapshotId, name: &QualifiedName) -> Result<Vec<ContentHash>, CodedError>;
        fn remaining_references(&self, snapshot: SnapshotId, name: &QualifiedName, simple: &SymbolName) -> Result<u32, CodedError>;
        fn same_simple_name(&self, snapshot: SnapshotId, simple: &SymbolName) -> Result<Vec<SameName>, CodedError>;
    }
}

impl CodeStatistics for SqliteStore {
    forward! { CodeStatistics via of_snapshot:
        fn symbol_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFacts>, CodedError>;
        fn prepare(&self, snapshot: SnapshotId, setup: StatisticsSetup<'_>) -> Result<(), CodedError>;
        fn unused_counts(&self, snapshot: SnapshotId, query: &RankingQuery, setup: StatisticsSetup<'_>) -> Result<UnusedCounts, CodedError>;
        fn summary(&self, snapshot: SnapshotId, include_tests: bool, setup: StatisticsSetup<'_>) -> Result<StatisticsSummary, CodedError>;
    }

    fn ranked(&self, request: &RankingRequest<'_>) -> Result<RankedRows, CodedError> {
        CodeStatistics::ranked(&*self.of_snapshot(request.snapshot)?, request)
    }
}

impl ConsentStore for SqliteStore {
    forward! { ConsentStore via of_project:
        fn consent(&self, project: ProjectId) -> Result<ConsentRecord, CodedError>;
    }

    fn store_consent(&self, grant: &ConsentGrant) -> Result<(), CodedError> {
        ConsentStore::store_consent(&*self.of_project(grant.project())?, grant)
    }
}

impl ScopeStore for SqliteStore {
    fn store_scope(&self, project: ProjectId, scope: &Scope) -> Result<(), CodedError> {
        ScopeStore::store_scope(&*self.of_project(project)?, project, scope)
    }
}

impl ToolReportStore for SqliteStore {
    forward! { ToolReportStore via of_project:
        fn load(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError>;
        fn replace(&self, project: ProjectId, report: &ToolReport) -> Result<(), CodedError>;
    }
}

impl ScanReportStore for SqliteStore {
    forward! { ScanReportStore via of_project:
        fn report(&self, project: ProjectId) -> Result<Option<ScanReport>, CodedError>;
    }

    fn store_report(&self, write: &ReportWrite) -> Result<(), CodedError> {
        ScanReportStore::store_report(&*self.of_project(write.project)?, write)
    }
}

impl ProjectCatalog for SqliteStore {
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        let mut all = Vec::new();
        for entry in self.entries().map_err(CodedError::from)? {
            let Ok(db) = self.db_any(entry.project) else {
                all.push(
                    self.unreadable_overview(entry.project)
                        .map_err(CodedError::from)?,
                );
                continue;
            };
            for mut overview in ProjectCatalog::overviews(&*db)? {
                overview.database = self.database_info(entry.project, entry.removing);
                all.push(overview);
            }
        }
        Ok(all)
    }

    forward! { ProjectCatalog via of_project:
        fn languages(&self, project: ProjectId) -> Result<Vec<LanguageShare>, CodedError>;
    }
}

impl AnnotationStore for SqliteStore {
    fn create_annotation(&self, creation: &AnnotationCreation) -> Result<Annotation, CodedError> {
        let db = self.of_project(creation.annotation.project)?;
        AnnotationStore::create_annotation(&*db, creation)
    }

    fn update_annotation(&self, update: &AnnotationUpdate) -> Result<Annotation, CodedError> {
        let db = self.of_project(update.annotation.project)?;
        AnnotationStore::update_annotation(&*db, update)
    }

    fn list_annotations(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>, CodedError> {
        // A project that does not exist has no annotations, as before: callers answer "not found".
        match self.db(filter.project) {
            Ok(db) => AnnotationStore::list_annotations(&*db, filter),
            Err(StorageError::ProjectNotFound) => Ok(Vec::new()),
            Err(other) => Err(other.into()),
        }
    }

    forward! { AnnotationStore via of_annotation:
        fn delete_annotation(&self, id: AnnotationId) -> Result<(), CodedError>;
        fn baseline(&self, annotation: AnnotationId) -> Result<Option<SymbolBaseline>, CodedError>;
        fn store_baseline(&self, annotation: AnnotationId, baseline: &SymbolBaseline) -> Result<(), CodedError>;
    }

    fn refresh_targets(&self, project: ProjectId, snapshot: SnapshotId) -> Result<u64, CodedError> {
        AnnotationStore::refresh_targets(&*self.of_project(project)?, project, snapshot)
    }

    fn raise_alert(&self, raise: &AlertRaise) -> Result<Alert, CodedError> {
        AnnotationStore::raise_alert(&*self.of_annotation(raise.annotation)?, raise)
    }

    fn list_alerts(&self, filter: &AlertFilter) -> Result<Vec<Alert>, CodedError> {
        match self.db(filter.project) {
            Ok(db) => AnnotationStore::list_alerts(&*db, filter),
            Err(StorageError::ProjectNotFound) => Ok(Vec::new()),
            Err(other) => Err(other.into()),
        }
    }

    fn dismiss_alert(&self, dismissal: &AlertDismissal) -> Result<Alert, CodedError> {
        AnnotationStore::dismiss_alert(&*self.of_alert(dismissal.alert)?, dismissal)
    }

    fn purge_dismissed(&self, purge: &AlertPurge) -> Result<u64, CodedError> {
        let mut deleted = 0;
        for entry in self.entries().map_err(CodedError::from)? {
            if entry.removing {
                continue;
            }
            // One file that does not open must not stop the sweep over the others.
            match self
                .of_project(entry.project)
                .and_then(|db| AnnotationStore::purge_dismissed(&*db, purge))
            {
                Ok(count) => deleted += count,
                Err(error) => tracing::warn!(
                    event = LOG_PROJECT_SKIPPED,
                    project = entry.project.get(),
                    error = %error
                ),
            }
        }
        Ok(deleted)
    }
}
