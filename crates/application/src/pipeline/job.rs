//! The lifecycle of a scan job as a typestate: `Job<Queued>` → `Job<Running>` → one of
//! `Job<Finished>`, `Job<Cancelled>` or `Job<Failed>`. A transition that makes no sense does not
//! compile, because each method exists only in the state where it is allowed.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ProjectId, ScanMode, Snapshot};

use crate::cancel::CancelToken;

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

    fn into_state<T: JobState>(self, state: T) -> Job<T> {
        Job {
            project: self.project,
            mode: self.mode,
            cancel: self.cancel,
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
            state: Queued,
        }
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

    /// Ends the job with its snapshot.
    #[must_use]
    pub fn finish(self, snapshot: Snapshot) -> Job<Finished> {
        self.into_state(Finished { snapshot })
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
}

impl Job<Failed> {
    /// The code of the failure.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.state.code
    }
}
