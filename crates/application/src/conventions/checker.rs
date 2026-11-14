//! The use case: discover, analyse, check.

use std::collections::HashSet;
use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, RelativePath, SnapshotId};

use super::duplicates::{Fingerprinted, duplicates};
use super::length::{Callable, callables, too_long};
use super::model::{CheckReport, CheckRequest, Finding, StoredProject, Unchecked};
use super::selection::Selection;
use super::stored::{Rules, against_stored};
use crate::access;
use crate::cancel::CancelToken;
use crate::config::Conventions;
use crate::consts::{CHECK_FIRST_ID, CHECK_SNAPSHOT_ID, PATH_SEPARATOR};
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts};

/// The rules with a `CHK` code that this version cannot check yet. Literals and cognitive
/// complexity need counts the language packs do not deliver; layering needs the allowed
/// directions, which the project model does not carry (issue #45).
const NOT_CHECKED: &[ErrorCode] = &[
    ErrorCode::LiteralFound,
    ErrorCode::LayerViolation,
    ErrorCode::ComplexityTooHigh,
];

/// A file after analysis, with the callables it declares.
struct Analysed {
    path: RelativePath,
    callables: Vec<Callable>,
    facts: FileFacts,
}

/// Checks files against the code rules of blueprint section 3.
pub struct ConventionChecker {
    discovery: Arc<dyn FileDiscovery>,
    analyzer: Arc<dyn FileAnalyzer>,
    limits: Conventions,
}

impl ConventionChecker {
    /// Builds the checker over the same ports a scan uses. The analyzer must have fingerprints
    /// switched on, otherwise no duplicate is found.
    #[must_use]
    pub fn new(
        discovery: Arc<dyn FileDiscovery>,
        analyzer: Arc<dyn FileAnalyzer>,
        limits: Conventions,
    ) -> Self {
        Self {
            discovery,
            analyzer,
            limits,
        }
    }

    /// Analyses the selected files and reports every violation.
    ///
    /// Files that cannot be analysed (a language without a pack, a parse timeout) do not stop the
    /// check; they are listed in `unchecked`. Nothing is stored.
    ///
    /// # Errors
    /// - `AUTH-401` if the caller may not read source.
    /// - A coded failure of the file discovery, for example `SRC-201` or `SRC-501`.
    pub fn check(&self, request: &CheckRequest<'_>) -> Result<CheckReport, CodedError> {
        access::require(request.principal, Permission::ReadSource)?;
        let selection = Selection::new(request.paths);
        let files = self
            .discovery
            .discover(request.root, &[], &CancelToken::new())?;
        let ids = IdSequence::starting_at(CHECK_FIRST_ID);
        let mut analysed = Vec::new();
        let mut unchecked = Vec::new();
        let wanted = |file: &&DiscoveredFile| {
            selection.contains(&file.path) && !in_test_folder(&file.path, &self.limits.test_folders)
        };
        for file in files.iter().filter(wanted) {
            match self.analyse(request, file, &ids) {
                Ok(done) => analysed.push(done),
                Err(error) => unchecked.push(Unchecked {
                    path: file.path.clone(),
                    code: error.code(),
                }),
            }
        }
        let unmatched = selection.unmatched(&files);
        self.report(&analysed, request.stored.as_ref(), (unchecked, unmatched))
    }

    fn analyse(
        &self,
        request: &CheckRequest<'_>,
        file: &DiscoveredFile,
        ids: &IdSequence,
    ) -> Result<Analysed, CodedError> {
        let facts = self.analyzer.analyze(&AnalyzeRequest {
            root: request.root,
            snapshot: SnapshotId::new(CHECK_SNAPSHOT_ID),
            file,
            ids,
        })?;
        Ok(Analysed {
            path: file.path.clone(),
            callables: callables(&file.path, &facts),
            facts,
        })
    }

    fn report(
        &self,
        analysed: &[Analysed],
        stored: Option<&StoredProject<'_>>,
        (unchecked, unmatched): (Vec<Unchecked>, Vec<String>),
    ) -> Result<CheckReport, CodedError> {
        let mut findings: Vec<Finding> = analysed
            .iter()
            .flat_map(|done| too_long(&done.callables, self.limits.max_function_lines))
            .collect();
        let items = fingerprinted(analysed);
        let inside = duplicates(
            &items,
            (
                self.limits.duplicate_similarity_percent,
                self.limits.duplicate_min_lines,
            ),
            self.limits.duplicate_max_owners,
        );
        if let Some(stored) = stored {
            findings.extend(self.outside((analysed, &items), stored, &inside)?);
        }
        findings.extend(inside);
        findings.sort_by_key(|finding| {
            (
                finding.location.path.clone(),
                finding.location.span.start(),
                finding.code.id(),
            )
        });
        Ok(CheckReport {
            files_checked: u32::try_from(analysed.len()).unwrap_or(u32::MAX),
            findings,
            unchecked,
            not_checked: NOT_CHECKED.to_vec(),
            unmatched,
        })
    }

    /// Copies of stored code outside the checked files, for the functions `inside` did not report.
    fn outside(
        &self,
        (analysed, items): (&[Analysed], &[Fingerprinted<'_>]),
        stored: &StoredProject<'_>,
        inside: &[Finding],
    ) -> Result<Vec<Finding>, CodedError> {
        let checked: HashSet<RelativePath> =
            analysed.iter().map(|done| done.path.clone()).collect();
        let folders = &self.limits.test_folders;
        let is_test = |path: &RelativePath| in_test_folder(path, folders);
        let rules = Rules {
            minimum_percent: self.limits.duplicate_similarity_percent,
            minimum_lines: self.limits.duplicate_min_lines,
            checked: &checked,
            is_test: &is_test,
        };
        against_stored(items, stored, &rules, inside)
    }
}

/// Whether a folder on the way to `path` is one of the test folders.
fn in_test_folder(path: &RelativePath, test_folders: &[String]) -> bool {
    let mut parts: Vec<&str> = path.as_str().split(PATH_SEPARATOR).collect();
    parts.pop();
    parts
        .iter()
        .any(|part| test_folders.iter().any(|folder| folder == part))
}

/// The callables that have fingerprints, paired with them.
fn fingerprinted(analysed: &[Analysed]) -> Vec<Fingerprinted<'_>> {
    analysed
        .iter()
        .flat_map(|done| {
            done.callables.iter().filter_map(|callable| {
                let found = done
                    .facts
                    .fingerprints
                    .iter()
                    .find(|entry| entry.symbol == callable.id)?;
                Some(Fingerprinted {
                    callable,
                    fingerprints: &found.fingerprints,
                })
            })
        })
        .collect()
}
