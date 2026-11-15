//! The lifecycle of a scan job as a typestate: `Job<Queued>` → `Job<Running>` → one of
//! `Job<Finished>`, `Job<Cancelled>` or `Job<Failed>`. A transition that makes no sense does not
//! compile, because each method exists only in the state where it is allowed.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ProjectId, ScanMode, Snapshot};

use crate::cancel::CancelToken;
use crate::level2::LanguageReport;
use crate::ports::ScanWarning;
use crate::selection::Selection;

mod sealed {
    /// Keeps the set of job states closed.
    pub trait Sealed {}
}

/// Marks the states a job can be in. Closed: only the five states of this module exist.
pub trait JobState: sealed::Sealed {}

/// Waiting to be started.
#[derive(Debug)]
pub struct Queued;
/// Currently running.
#[derive(Debug)]
pub struct Running;
/// Ended with a snapshot.
#[derive(Debug)]
pub struct Finished {
    snapshot: Snapshot,
    level2: Vec<LanguageReport>,
    warnings: Vec<ScanWarning>,
}
/// Ended because the user cancelled; no snapshot exists.
#[derive(Debug)]
pub struct Cancelled;
/// Ended because a stage failed.
#[derive(Debug)]
pub struct Failed {
    code: ErrorCode,
}

macro_rules! job_states {
    ($($state:ident),+) => {
        $(
            impl sealed::Sealed for $state {}
            impl JobState for $state {}
        )+
    };
}
job_states!(Queued, Running, Finished, Cancelled, Failed);

/// One scan of one project in state `S`.
#[derive(Debug)]
pub struct Job<S: JobState> {
    project: ProjectId,
    mode: ScanMode,
    cancel: CancelToken,
    level2: bool,
    paths: Selection,
    state: S,
}

impl<S: JobState> Job<S> {
    /// The project being scanned.
    #[must_use]
    pub const fn project(&self) -> ProjectId {
        self.project
    }

    /// Full or incremental.
    #[must_use]
    pub const fn mode(&self) -> ScanMode {
        self.mode
    }

    /// Whether level 2 runs after level 1 is stored; false unless the job was built
    /// `with_level2`.
    #[must_use]
    pub const fn level2_wanted(&self) -> bool {
        self.level2
    }

    /// The paths an incremental scan is restricted to; none means the whole project.
    #[must_use]
    pub const fn paths(&self) -> &Selection {
        &self.paths
    }

    fn into_state<T: JobState>(self, state: T) -> Job<T> {
        Job {
            project: self.project,
            mode: self.mode,
            cancel: self.cancel,
            level2: self.level2,
            paths: self.paths,
            state,
        }
    }
}

impl Job<Queued> {
    /// Queues a scan of `project`.
    #[must_use]
    pub fn new(project: ProjectId, mode: ScanMode) -> Self {
        Self {
            project,
            mode,
            cancel: CancelToken::new(),
            level2: false,
            paths: Selection::default(),
            state: Queued,
        }
    }

    /// The same job with the second pass: after level 1 it may start the indexers the project
    /// has consent for. A job does not want this by default, so that MCP, the UI and the server
    /// never start an indexer by accident; only the scan of the command line asks for it
    /// (ADR 0033, decisions 5 and 13).
    #[must_use]
    pub const fn with_level2(mut self) -> Self {
        self.level2 = true;
        self
    }

    /// The same job without the second pass: level 1 only. Watch mode uses it for the scans after
    /// the first, so that no indexer starts on every change of a file (ADR 0033, decision 13).
    #[must_use]
    pub const fn without_level2(mut self) -> Self {
        self.level2 = false;
        self
    }

    /// The same job restricted to the files and folders `paths` name: an incremental scan then
    /// looks only there for changes, and every other file of the previous snapshot stays as it
    /// is, deleted or not. A full scan ignores the limit. No path means the whole project.
    #[must_use]
    pub fn with_paths(mut self, paths: &[String]) -> Self {
        self.paths = Selection::new(paths);
        self
    }

    /// A handle that cancels the job; take it before calling `start`.
    #[must_use]
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel.clone()
    }

    /// Moves the job to running.
    #[must_use]
    pub fn start(self) -> Job<Running> {
        self.into_state(Running)
    }

    /// Ends the job before it ran.
    #[must_use]
    pub fn cancel(self) -> Job<Cancelled> {
        self.into_state(Cancelled)
    }
}

impl Job<Running> {
    /// The token that stages poll.
    #[must_use]
    pub const fn token(&self) -> &CancelToken {
        &self.cancel
    }

    /// Ends the job with its snapshot, what the second pass reports per language and the
    /// warnings of the scan.
    #[must_use]
    pub fn finish(
        self,
        snapshot: Snapshot,
        level2: Vec<LanguageReport>,
        warnings: Vec<ScanWarning>,
    ) -> Job<Finished> {
        self.into_state(Finished {
            snapshot,
            level2,
            warnings,
        })
    }

    /// Ends the job because the user cancelled.
    #[must_use]
    pub fn cancel(self) -> Job<Cancelled> {
        self.into_state(Cancelled)
    }

    /// Ends the job because a stage failed with `code`.
    #[must_use]
    pub fn fail(self, code: ErrorCode) -> Job<Failed> {
        self.into_state(Failed { code })
    }
}

impl Job<Finished> {
    /// The snapshot the scan created.
    #[must_use]
    pub const fn snapshot(&self) -> &Snapshot {
        &self.state.snapshot
    }

    /// What level 2 did for each language; empty if it is not wired.
    #[must_use]
    pub fn level2(&self) -> &[LanguageReport] {
        &self.state.level2
    }

    /// The warnings of the scan, one per code with a count; empty if there is none.
    #[must_use]
    pub fn warnings(&self) -> &[ScanWarning] {
        &self.state.warnings
    }
}

impl Job<Failed> {
    /// The code of the failure.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.state.code
    }
}
