//! The answer that the screen, the command and the tool give: every rule with its violations,
//! judged against the held state (ADR 0104).

use codeskimr_domain::{Layer, SnapshotId};

use super::baseline::{Baseline, BaselineEntry};
use super::evaluate::evaluate;
use super::model::Outcome;
use crate::config::Rules;
use crate::error::CodedError;
use crate::ports::ReferencePlaces;

/// What the rules of a project say about the newest scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesReport {
    /// The day the state was held, `None` if it never was.
    pub held_on: Option<String>,
    /// Every rule, with its violations marked old or new.
    pub outcomes: Vec<Outcome>,
    /// Held violations that are gone.
    pub resolved: Vec<BaselineEntry>,
}

impl RulesReport {
    /// How many violations are old burdens.
    #[must_use]
    pub fn old_burdens(&self) -> usize {
        self.outcomes
            .iter()
            .map(|outcome| outcome.violations.len() - outcome.new_violations())
            .sum()
    }

    /// How many violations are not held: the number that fails the CI.
    #[must_use]
    pub fn new_violations(&self) -> usize {
        self.outcomes.iter().map(Outcome::new_violations).sum()
    }
}

/// Evaluates the `layers` and `rules` over `snapshot` and judges the result against `baseline`, if there is one.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn report(
    rules: &Rules,
    layers: &[Layer],
    baseline: Option<&Baseline>,
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
) -> Result<RulesReport, CodedError> {
    let outcomes = evaluate(rules, layers, references, snapshot)?;
    Ok(match baseline {
        Some(held) => {
            let judged = held.judge(outcomes);
            RulesReport {
                held_on: Some(held.held_on.clone()),
                outcomes: judged.outcomes,
                resolved: judged.resolved,
            }
        }
        None => RulesReport {
            held_on: None,
            outcomes,
            resolved: Vec::new(),
        },
    })
}
