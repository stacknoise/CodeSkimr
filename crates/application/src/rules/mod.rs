//! The architecture rules of a project and the held state of their old violations (ADR 0104 to
//! 0107): what the screen "Rules", `codeskimr rules` and the MCP tool read.

mod baseline;
mod consts;
mod cycles;
mod evaluate;
mod model;
mod port;
mod report;

pub use baseline::{Baseline, BaselineEntry, Judged};
pub use consts::BASELINE_FILE;
pub use evaluate::evaluate;
pub use model::{Outcome, Place, Ring, RuleKind, Violation};
pub use port::RulesChecks;
pub use report::{RulesReport, report};
