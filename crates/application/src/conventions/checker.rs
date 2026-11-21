//! The use case: discover, analyse, check.

use std::collections::HashSet;
use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{FolderPath, Permission, RelativePath, SnapshotId};

use super::allowed::is_allowed;
use super::duplicates::{Fingerprinted, duplicates};
use super::layering::{can_be_checked, layering};
use super::length::{
    Callable, callables, too_complex, too_long, too_many_parameters, with_literals,
};
use super::model::{CheckReport, CheckRequest, Finding, StoredProject, Unchecked};
use super::stored::{Rules, against_stored};
use crate::access;
use crate::cancel::CancelToken;
use crate::config::Conventions;
use crate::consts::{CHECK_FIRST_ID, CHECK_SNAPSHOT_ID, PATH_SEPARATOR};
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{
    AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts, SourceText,
};
use crate::selection::Selection;

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
    text: Arc<dyn SourceText>,
    limits: Conventions,
}

impl ConventionChecker {
    /// Builds the checker over the same ports a scan uses. The analyzer must have fingerprints
    /// switched on, otherwise no duplicate is found. `text` is read for the marker that exempts a
    /// pure mapping function from `CHK-102` (ADR 0070).
    #[must_use]
    pub fn new(
        discovery: Arc<dyn FileDiscovery>,
        analyzer: Arc<dyn FileAnalyzer>,
        text: Arc<dyn SourceText>,
        limits: Conventions,
    ) -> Self {
        Self {
            discovery,
            analyzer,
            text,
            limits,
        }
    }

    /// The limits this checker applies.
    #[must_use]
    pub fn limits(&self) -> &Conventions {
        &self.limits
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
        self.report(
            &analysed,
            (request.stored.as_ref(), request.root),
            (unchecked, unmatched),
        )
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
        (stored, root): (Option<&StoredProject<'_>>, &FolderPath),
        (unchecked, unmatched): (Vec<Unchecked>, Vec<String>),
    ) -> Result<CheckReport, CodedError> {
        let mut findings: Vec<Finding> = analysed
            .iter()
            .flat_map(|done| self.measured(&done.callables))
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
            let files: Vec<RelativePath> = analysed.iter().map(|done| done.path.clone()).collect();
            findings.extend(layering(stored, &files)?);
        }
        findings.extend(inside);
        findings.retain(|finding| !self.allowed(analysed, root, finding));
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
            not_checked: not_checked(analysed, stored),
            unmatched,
        })
    }

    /// Whether `finding` is a copy that its function excuses with the marker (ADR 0070).
    fn allowed(&self, analysed: &[Analysed], root: &FolderPath, finding: &Finding) -> bool {
        finding.code == ErrorCode::DuplicateBlock
            && analysed
                .iter()
                .flat_map(|done| &done.callables)
                .find(|callable| callable.location == finding.location)
                .is_some_and(|callable| is_allowed(self.text.as_ref(), root, callable))
    }

    /// The findings about length, cognitive complexity and parameters.
    fn measured(&self, callables: &[Callable]) -> Vec<Finding> {
        let mut findings = too_long(callables, self.limits.max_function_lines);
        findings.extend(too_complex(callables, self.limits.max_cognitive_complexity));
        findings.extend(too_many_parameters(callables, self.limits.max_parameters));
        findings.extend(with_literals(callables));
        findings
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

/// The rules this check did not (fully) check: the layering without a stored project that sets
/// limits between its layers (ADR 0082), and the cognitive complexity, the parameters and the
/// literals when a function of a language whose pack does not report them was among the files.
fn not_checked(analysed: &[Analysed], stored: Option<&StoredProject<'_>>) -> Vec<ErrorCode> {
    let mut rules = Vec::new();
    if !stored.is_some_and(|stored| can_be_checked(stored.layers)) {
        rules.push(ErrorCode::LayerViolation);
    }
    let all = || analysed.iter().flat_map(|done| &done.callables);
    if all().any(|callable| callable.cognitive.is_none()) {
        rules.push(ErrorCode::ComplexityTooHigh);
    }
    if all().any(|callable| callable.parameters.is_none()) {
        rules.push(ErrorCode::TooManyParameters);
    }
    if all().any(|callable| callable.literals.is_none()) {
        rules.push(ErrorCode::LiteralFound);
    }
    rules
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
