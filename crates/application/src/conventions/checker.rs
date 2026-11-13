//! The use case: discover, analyse, check.

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, SnapshotId};

use super::duplicates::{Fingerprinted, duplicates};
use super::length::{Callable, callables, too_long};
use super::model::{CheckReport, CheckRequest, Finding, Unchecked};
use super::selection::Selection;
use crate::access;
use crate::cancel::CancelToken;
use crate::config::Conventions;
use crate::consts::{CHECK_FIRST_ID, CHECK_SNAPSHOT_ID};
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
        for file in files.iter().filter(|file| selection.contains(&file.path)) {
            match self.analyse(request, file, &ids) {
                Ok(done) => analysed.push(done),
                Err(error) => unchecked.push(Unchecked {
                    path: file.path.clone(),
                    code: error.code(),
                }),
            }
        }
        let unmatched = selection.unmatched(&files);
        Ok(self.report(&analysed, unchecked, unmatched))
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
            callables: callables(&file.path, &facts),
            facts,
        })
    }

    fn report(
        &self,
        analysed: &[Analysed],
        unchecked: Vec<Unchecked>,
        unmatched: Vec<String>,
    ) -> CheckReport {
        let mut findings: Vec<Finding> = analysed
            .iter()
            .flat_map(|done| too_long(&done.callables, self.limits.max_function_lines))
            .collect();
        let items = fingerprinted(analysed);
        findings.extend(duplicates(&items, self.limits.duplicate_similarity_percent));
        findings.sort_by_key(|finding| {
            (
                finding.location.path.clone(),
                finding.location.span.start(),
                finding.code.id(),
            )
        });
        CheckReport {
            files_checked: u32::try_from(analysed.len()).unwrap_or(u32::MAX),
            findings,
            unchecked,
            not_checked: NOT_CHECKED.to_vec(),
            unmatched,
        }
    }
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
