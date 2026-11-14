//! The lifecycle of a scan job as a typestate: `Job<Queued>` → `Job<Running>` → one of
//! `Job<Finished>`, `Job<Cancelled>` or `Job<Failed>`. A transition that makes no sense does not
//! compile, because each method exists only in the state where it is allowed.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ProjectId, ScanMode, Snapshot};

use crate::cancel::CancelToken;
use crate::level2::LanguageReport;

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

    /// Whether level 2 runs after level 1 is stored; true unless the job was built
    /// `without_level2`.
    #[must_use]
    pub const fn level2_wanted(&self) -> bool {
        self.level2
    }

    fn into_state<T: JobState>(self, state: T) -> Job<T> {
        Job {
            project: self.project,
            mode: self.mode,
            cancel: self.cancel,
            level2: self.level2,
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
            level2: true,
            state: Queued,
        }
    }

    /// The same job without the second pass: level 1 only. Watch mode uses it for the scans after
    /// the first, so that no indexer starts on every change of a file (ADR 0033, decision 13).
    #[must_use]
    pub const fn without_level2(mut self) -> Self {
        self.level2 = false;
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

    /// Ends the job with its snapshot and what the second pass reports per language.
    #[must_use]
    pub fn finish(self, snapshot: Snapshot, level2: Vec<LanguageReport>) -> Job<Finished> {
        self.into_state(Finished { snapshot, level2 })
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
}

impl Job<Failed> {
    /// The code of the failure.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.state.code
    }
}
