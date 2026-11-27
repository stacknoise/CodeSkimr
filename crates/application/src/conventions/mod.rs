//! The convention check of blueprint 10.4 (`check_conventions`): reads the code the way a scan
//! does and reports where it breaks the code rules of blueprint section 3.
//!
//! The check analyses the selected files afresh, so it sees edits that no scan has seen yet. Only
//! the layer rule (`CHK-103`) reads the stored references of the newest snapshot, so it does not
//! see unscanned edits. The report lists what it does not check in `not_checked`.

mod allowed;
pub(crate) mod checker;
mod duplicates;
mod layering;
mod length;
mod model;
mod stored;

pub use checker::ConventionChecker;
pub use model::{
    CheckReport, CheckRequest, Detail, Finding, Location, Measure, StoredProject, Unchecked,
};
