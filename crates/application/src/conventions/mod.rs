//! The convention check of blueprint 10.4 (`check_conventions`): reads the code the way a scan
//! does and reports where it breaks the code rules of blueprint section 3.
//!
//! The check analyses the selected files afresh, so it sees edits that no scan has seen yet. What
//! it checks today: function length (`CHK-104`, R-02) and duplicated functions (`CHK-102`, R-01).
//! The report lists what it does not check yet in `not_checked`.

mod checker;
mod duplicates;
mod length;
mod model;
mod selection;
mod stored;

pub use checker::ConventionChecker;
pub use model::{CheckReport, CheckRequest, Detail, Finding, Location, StoredProject, Unchecked};
