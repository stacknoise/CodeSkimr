//! The commands of the scan history and of the comparison of two scans (ADR 0097): the list of the
//! last scans, the comparison of two of them and the impact of the difference. The comparison is
//! one use case, `codeskimr_application::compare::ScanComparer`; the commands only choose how much
//! of its answer to ask for. The Markdown of the impact is made by the same function as the one of
//! `codeskimr change --markdown` (`codeskimr_cli::change_markdown`), so the copy button of the
//! screen and the command line give the same text.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::compare::ScanListAnswer;
use codeskimr_application::compare::{
    ComparisonAnswer, ComparisonRequest, MetricsAnswer, MetricsRequest, MetricsWanted, ScanDiffs,
    metrics_request,
};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::{MetricsSide, change_markdown, metrics_markdown};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, SnapshotId};
use codeskimr_i18n::{DEFAULT_LANGUAGE, I18nError, Messages};
use serde::{Deserialize, Serialize};

use super::consts::{
    SCANS_COMPARE, SCANS_COMPARE_DESCRIPTION, SCANS_COMPARE_TITLE, SCANS_IMPACT,
    SCANS_IMPACT_DESCRIPTION, SCANS_IMPACT_MARKDOWN, SCANS_IMPACT_MARKDOWN_DESCRIPTION,
    SCANS_IMPACT_MARKDOWN_TITLE, SCANS_IMPACT_TITLE, SCANS_LIST, SCANS_LIST_DESCRIPTION,
    SCANS_LIST_TITLE, SCANS_METRICS, SCANS_METRICS_DESCRIPTION, SCANS_METRICS_MARKDOWN,
    SCANS_METRICS_MARKDOWN_DESCRIPTION, SCANS_METRICS_MARKDOWN_TITLE, SCANS_METRICS_TITLE,
};
use super::preferences::PreferencesFile;

/// The input of `scans.list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ListInput {
    /// Id of the project.
    pub project: u64,
}

/// The input of `scans.compare` and `scans.impact`. Without `base` and `target` the newest scan is
/// compared with the one before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct CompareInput {
    /// Id of the project.
    pub project: u64,
    /// Snapshot id of the older scan; the scan before the target if left out.
    #[serde(default)]
    pub base: Option<u64>,
    /// Snapshot id of the newer scan; the newest scan if left out.
    #[serde(default)]
    pub target: Option<u64>,
}

/// What the command reads.
pub struct ScanListPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    /// `snapshots.retention_count`.
    pub(super) keep: u32,
}

/// Lists the retained scans of a project, newest first.
pub struct ScansList {
    pub(super) ports: Arc<ScanListPorts>,
}

impl Command for ScansList {
    type Input = ListInput;
    type Output = ScanListAnswer;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCANS_LIST),
        title_key: SCANS_LIST_TITLE,
        description_key: SCANS_LIST_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ScanListAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let project = ProjectId::new(request.input.project);
        self.ports.projects.project(project)?;
        let history = self.ports.index.scan_history(project)?;
        Ok(ScanListAnswer::new(&history, self.ports.keep))
    }
}

/// What the commands of the comparison read.
pub struct ComparePorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) comparer: Arc<dyn ScanDiffs>,
}

/// The answer of the comparison, with or without the usages the difference hits.
fn answer(
    ports: &ComparePorts,
    input: &CompareInput,
    with_impact: bool,
) -> Result<ComparisonAnswer, CodedError> {
    let project = ProjectId::new(input.project);
    ports.projects.project(project)?;
    let comparison = ports.comparer.compare(&ComparisonRequest {
        project,
        base: input.base.map(SnapshotId::new),
        target: input.target.map(SnapshotId::new),
        with_impact,
    })?;
    Ok(ComparisonAnswer::new(&comparison, None))
}

/// The commands of the comparison. With `IMPACT` false (`scans.compare`) it compares two retained
/// scans: the symbols that were added, removed, changed or moved, the new layer violations and
/// cycles and the critical symbols the difference touches; `impact` is `null`. With `IMPACT` true
/// (`scans.impact`) it gives the same answer and, in `impact`, the direct usages of every removed
/// symbol and of every symbol whose signature or visibility changed, as adapted or not touched; one
/// call serves both tabs of the screen "Change".
pub struct ScansDiff<const IMPACT: bool> {
    pub(super) ports: Arc<ComparePorts>,
}

/// `scans.compare`: the comparison without the usages.
pub type ScansCompare = ScansDiff<false>;
/// `scans.impact`: the comparison with the usages the difference hits.
pub type ScansImpact = ScansDiff<true>;

impl<const IMPACT: bool> Command for ScansDiff<IMPACT> {
    type Input = CompareInput;
    type Output = ComparisonAnswer;

    const DESCRIPTOR: CommandDescriptor = if IMPACT {
        diff_descriptor(SCANS_IMPACT, SCANS_IMPACT_TITLE, SCANS_IMPACT_DESCRIPTION)
    } else {
        diff_descriptor(
            SCANS_COMPARE,
            SCANS_COMPARE_TITLE,
            SCANS_COMPARE_DESCRIPTION,
        )
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ComparisonAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        answer(&self.ports, &request.input, IMPACT)
    }
}

/// The descriptor of a read-only command of the comparison.
const fn diff_descriptor(
    id: &'static str,
    title_key: &'static str,
    description_key: &'static str,
) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key,
        description_key,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    }
}

/// The input of `scans.metrics`: the scans as for `scans.compare` and what to look at (ADR 0114).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetricsInput {
    /// The scans to compare, as for `scans.compare`.
    #[serde(flatten)]
    pub compare: CompareInput,
    /// `module`, `type` or `member`; `member` if left out.
    #[serde(default)]
    pub level: Option<String>,
    /// Whether test code is looked at too; `false` if left out.
    #[serde(default)]
    pub include_tests: bool,
    /// Only objects in which one of these metrics moved: `loc`, `cc`, `mi`, `ce` or
    /// `instability`; every judged metric if left out.
    #[serde(default)]
    pub metrics: Vec<String>,
}

impl MetricsInput {
    /// The request of the use case; `RequestInvalid` for a level or a metric that is not known.
    fn request(&self) -> Result<MetricsRequest, CodedError> {
        metrics_request(
            ProjectId::new(self.compare.project),
            (
                self.compare.base.map(SnapshotId::new),
                self.compare.target.map(SnapshotId::new),
            ),
            MetricsWanted {
                level: self.level.as_deref(),
                only: &self.metrics,
                include_tests: self.include_tests,
            },
        )
    }
}

/// `scans.metrics`: which objects got worse and which got better by their metrics between two
/// retained scans (ADR 0114); the same answer as `codeskimr compare --metrics --json`.
pub struct ScansMetrics {
    pub(super) ports: Arc<ComparePorts>,
}

impl Command for ScansMetrics {
    type Input = MetricsInput;
    type Output = MetricsAnswer;

    const DESCRIPTOR: CommandDescriptor = diff_descriptor(
        SCANS_METRICS,
        SCANS_METRICS_TITLE,
        SCANS_METRICS_DESCRIPTION,
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<MetricsAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let wanted = request.input.request()?;
        self.ports.projects.project(wanted.project)?;
        let found = self.ports.comparer.compare_metrics(&wanted)?;
        Ok(MetricsAnswer::new(&found))
    }
}

/// The input of `scans.impact_markdown`: that of `scans.impact` and the language of the text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MarkdownInput {
    /// The scans to compare, as for `scans.impact`.
    #[serde(flatten)]
    pub compare: CompareInput,
    /// Language of the text; the one of the stored preferences, or English, if left out.
    #[serde(default)]
    pub language: Option<String>,
}

/// The answer of `scans.impact_markdown`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownAnswer {
    /// The text a comment on a pull request holds, as `codeskimr change --markdown` prints it.
    pub markdown: String,
}

/// What `scans.impact_markdown` reads: the comparison, the texts and the preferences of the user.
pub struct MarkdownPorts {
    pub(super) compare: Arc<ComparePorts>,
    pub(super) messages: Messages,
    pub(super) preferences: PreferencesFile,
}

/// `scans.impact_markdown`: the impact of `scans.impact` as the Markdown of `change --markdown`.
pub struct ScansImpactMarkdown {
    pub(super) ports: Arc<MarkdownPorts>,
}

impl Command for ScansImpactMarkdown {
    type Input = MarkdownInput;
    type Output = MarkdownAnswer;

    const DESCRIPTOR: CommandDescriptor = diff_descriptor(
        SCANS_IMPACT_MARKDOWN,
        SCANS_IMPACT_MARKDOWN_TITLE,
        SCANS_IMPACT_MARKDOWN_DESCRIPTION,
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<MarkdownAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let input = request.input;
        let answer = answer(&self.ports.compare, &input.compare, true)?;
        let value = serde_json::to_value(&answer)
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
        self.ports.markdown(input.language, |messages, language| {
            change_markdown(messages, language, &value)
        })
    }
}

impl MarkdownPorts {
    /// The text `make` builds, in the language asked for, the saved one or English.
    fn markdown(
        &self,
        language: Option<String>,
        make: impl FnOnce(&Messages, &str) -> Result<String, I18nError>,
    ) -> Result<MarkdownAnswer, CodedError> {
        let language = language
            .or_else(|| self.preferences.read().language)
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_owned());
        let markdown = make(&self.messages, &language).map_err(CodedError::caused_by)?;
        Ok(MarkdownAnswer { markdown })
    }
}

/// The input of `scans.metrics_markdown`: that of `scans.metrics` and the language of the text.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetricsMarkdownInput {
    /// The scans and what to look at, as for `scans.metrics`.
    #[serde(flatten)]
    pub metrics: MetricsInput,
    /// Language of the text; the one of the stored preferences, or English, if left out.
    #[serde(default)]
    pub language: Option<String>,
}

/// `scans.metrics_markdown`: the answer of `scans.metrics` as the Markdown of
/// `compare --metrics --markdown`.
pub struct ScansMetricsMarkdown {
    pub(super) ports: Arc<MarkdownPorts>,
}

impl Command for ScansMetricsMarkdown {
    type Input = MetricsMarkdownInput;
    type Output = MarkdownAnswer;

    const DESCRIPTOR: CommandDescriptor = diff_descriptor(
        SCANS_METRICS_MARKDOWN,
        SCANS_METRICS_MARKDOWN_TITLE,
        SCANS_METRICS_MARKDOWN_DESCRIPTION,
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<MarkdownAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let input = request.input;
        let wanted = input.metrics.request()?;
        self.ports.compare.projects.project(wanted.project)?;
        let found = self.ports.compare.comparer.compare_metrics(&wanted)?;
        let value = serde_json::to_value(MetricsAnswer::new(&found))
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
        self.ports.markdown(input.language, |messages, language| {
            metrics_markdown(messages, language, &value, MetricsSide::Both.lists())
        })
    }
}
