//! The generic pipeline engine: stages are chained at compile time, so a stage can only follow
//! one whose output type it accepts.

use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ProjectId;

use crate::cancel::CancelToken;
use crate::error::CodedError;
use crate::events::{Event, EventBus, ScanProgressed};
use crate::stage::StageKind;

/// Why a stage stopped before producing its output.
#[derive(Debug)]
pub enum Interruption {
    /// The cancel token was raised; the stage left nothing behind.
    Cancelled,
    /// The stage failed.
    Failed(CodedError),
}

impl From<CodedError> for Interruption {
    fn from(cause: CodedError) -> Self {
        Self::Failed(cause)
    }
}

/// A stage failure together with the stage it happened in. Its code is the code of the cause.
#[derive(Debug)]
pub struct PipelineError {
    /// Stage that failed.
    pub stage: StageKind,
    /// The failure reported by the stage.
    pub cause: CodedError,
}

impl Coded for PipelineError {
    fn code(&self) -> ErrorCode {
        self.cause.code()
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, f)
    }
}

impl Error for PipelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

/// Why a whole pipeline run ended without a result.
#[derive(Debug)]
pub enum Halt {
    /// Cancelled at or before this stage.
    Cancelled(StageKind),
    /// A stage failed.
    Failed(PipelineError),
}

/// What a stage can see and do while it runs.
#[derive(Debug, Clone, Copy)]
pub struct StageContext<'a> {
    project: ProjectId,
    stage: StageKind,
    cancel: &'a CancelToken,
    events: &'a EventBus,
    started: Instant,
    last: Option<&'a LastReport>,
}

impl<'a> StageContext<'a> {
    /// A context for `project`, starting at the first stage.
    #[must_use]
    pub fn new(project: ProjectId, cancel: &'a CancelToken, events: &'a EventBus) -> Self {
        Self {
            project,
            stage: StageKind::Fetch,
            cancel,
            events,
            started: Instant::now(),
            last: None,
        }
    }

    /// The context of `stage`, which remembers its last report in `last`.
    fn entering<'b>(self, stage: StageKind, last: &'b LastReport) -> StageContext<'b>
    where
        'a: 'b,
    {
        StageContext {
            project: self.project,
            stage,
            cancel: self.cancel,
            events: self.events,
            started: Instant::now(),
            last: Some(last),
        }
    }

    /// Publishes the last report of the stage again, now with the time the whole stage took.
    fn finished(&self) {
        let (done, total) = self.last.map_or((0, 0), |last| {
            (
                last.done.load(Ordering::Relaxed),
                last.total.load(Ordering::Relaxed),
            )
        });
        self.progress(done, total);
    }

    /// The token that tells long-running stages to stop.
    #[must_use]
    pub const fn cancel_token(&self) -> &'a CancelToken {
        self.cancel
    }

    /// Whether the scan was cancelled; stages check this between units of work.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Publishes `ScanProgressed`; pass `total` zero while the amount of work is unknown.
    pub fn progress(&self, done: u32, total: u32) {
        if let Some(last) = self.last {
            last.done.store(done, Ordering::Relaxed);
            last.total.store(total, Ordering::Relaxed);
        }
        self.events.publish(&Event::ScanProgressed(ScanProgressed {
            project: self.project,
            stage: self.stage,
            done,
            total,
            elapsed_ms: elapsed_ms(self.started),
        }));
    }
}

/// One filter of the pipeline. A stage is independent of the others and testable on its own.
pub trait Stage {
    /// What the stage consumes.
    type Input;
    /// What the stage produces.
    type Output;

    /// The stage this implementation fills.
    fn kind(&self) -> StageKind;

    /// Processes `input`, checking `ctx.is_cancelled()` between units of work.
    ///
    /// # Errors
    /// `Interruption::Cancelled` or `Interruption::Failed`.
    fn run(&self, input: Self::Input, ctx: &StageContext<'_>)
    -> Result<Self::Output, Interruption>;
}

/// A runnable chain of stages, from its `Input` to its `Output`.
pub trait Flow {
    /// What the first stage consumes.
    type Input;
    /// What the last stage produces.
    type Output;

    /// Runs all stages. Cancellation is checked before each stage starts.
    ///
    /// # Errors
    /// `Halt` when the run was cancelled or a stage failed.
    fn run(&self, input: Self::Input, ctx: &StageContext<'_>) -> Result<Self::Output, Halt>;
}

/// The last report of a stage: the units finished and the units in total.
#[derive(Debug, Default)]
struct LastReport {
    done: AtomicU32,
    total: AtomicU32,
}

/// Milliseconds since `started`, saturating, for the log and the events of a stage.
fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// A chain of one stage.
#[derive(Debug, Clone, Copy)]
pub struct Single<S>(S);

impl<S: Stage> Flow for Single<S> {
    type Input = S::Input;
    type Output = S::Output;

    fn run(&self, input: Self::Input, ctx: &StageContext<'_>) -> Result<Self::Output, Halt> {
        let kind = self.0.kind();
        if ctx.is_cancelled() {
            return Err(Halt::Cancelled(kind));
        }
        let last = LastReport::default();
        let ctx = ctx.entering(kind, &last);
        ctx.progress(0, 0);
        let started = Instant::now();
        let result = self.0.run(input, &ctx);
        tracing::info!(stage = kind.label(), elapsed_ms = elapsed_ms(started));
        if result.is_ok() {
            ctx.finished();
        }
        result.map_err(|stop| match stop {
            // An adapter that stops because of the token can only report an error; that is
            // still a cancellation, not a failure.
            Interruption::Cancelled => Halt::Cancelled(kind),
            Interruption::Failed(_) if ctx.is_cancelled() => Halt::Cancelled(kind),
            Interruption::Failed(cause) => Halt::Failed(PipelineError { stage: kind, cause }),
        })
    }
}

/// A chain followed by one more stage.
#[derive(Debug, Clone, Copy)]
pub struct Chain<F, S> {
    head: F,
    last: Single<S>,
}

impl<F, S> Flow for Chain<F, S>
where
    F: Flow,
    S: Stage<Input = F::Output>,
{
    type Input = F::Input;
    type Output = S::Output;

    fn run(&self, input: Self::Input, ctx: &StageContext<'_>) -> Result<Self::Output, Halt> {
        let middle = self.head.run(input, ctx)?;
        self.last.run(middle, ctx)
    }
}

/// A pipeline under construction or ready to run.
#[derive(Debug, Clone, Copy)]
pub struct Pipeline<F>(F);

impl<S: Stage> Pipeline<Single<S>> {
    /// Starts a pipeline with its first stage.
    #[must_use]
    pub const fn new(first: S) -> Self {
        Self(Single(first))
    }
}

impl<F: Flow> Pipeline<F> {
    /// Appends a stage; it must accept what the pipeline produces so far.
    #[must_use]
    pub fn then<S>(self, next: S) -> Pipeline<Chain<F, S>>
    where
        S: Stage<Input = F::Output>,
    {
        Pipeline(Chain {
            head: self.0,
            last: Single(next),
        })
    }

    /// Runs the pipeline.
    ///
    /// # Errors
    /// `Halt` when the run was cancelled or a stage failed.
    pub fn run(&self, input: F::Input, ctx: &StageContext<'_>) -> Result<F::Output, Halt> {
        self.0.run(input, ctx)
    }
}
