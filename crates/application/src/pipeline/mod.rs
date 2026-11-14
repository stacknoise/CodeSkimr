//! The scan pipeline (pipes and filters, blueprint section 7) and the typestate of a scan job.
//!
//! `flow` holds the generic engine, `stages` the six concrete stages built from the ports,
//! `job` the job lifecycle and `scanner` the entry point that ties them together.

mod carry;
mod flow;
mod job;
mod scanner;
mod stages;

pub use carry::Carried;
pub use flow::{
    Chain, Flow, Halt, Interruption, Pipeline, PipelineError, Single, Stage, StageContext,
};
pub use job::{Cancelled, Failed, Finished, Job, JobState, Queued, Running};
pub use scanner::{JobResult, Scanner, ScannerPorts};
pub use stages::{
    Analyzed, Discovered, Measured, Persisted, Resolved, ScanRequest, ScanRun, analyze_workers,
};
