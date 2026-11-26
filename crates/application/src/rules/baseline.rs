//! The held state: the violations that existed on a day and count as old burdens (ADR 0104).

use std::collections::BTreeSet;

use codeskimr_codes::ErrorCode;
use serde::{Deserialize, Serialize};

use super::model::Outcome;
use crate::error::CodedError;

/// One held violation. The line is not part of it, so moving code keeps it old.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineEntry {
    /// The id of the rule.
    pub rule: String,
    /// The referring symbol.
    pub from: String,
    /// The symbol it must not reach.
    pub to: String,
}

/// The content of `.codeskimr-baseline.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// The day the state was held, as the screen shows it.
    pub held_on: String,
    /// The held violations, in order.
    #[serde(default)]
    pub entries: Vec<BaselineEntry>,
}

/// The outcomes of the rules judged against the held state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judged {
    /// The outcomes, each violation marked old or new.
    pub outcomes: Vec<Outcome>,
    /// Held entries whose violation is gone: the burden went down.
    pub resolved: Vec<BaselineEntry>,
}

impl Baseline {
    /// The state that holds every violation of `outcomes` as of the day `held_on`.
    #[must_use]
    pub fn holding(held_on: &str, outcomes: &[Outcome]) -> Self {
        let entries: BTreeSet<BaselineEntry> = outcomes
            .iter()
            .flat_map(|outcome| {
                outcome.violations.iter().map(|violation| BaselineEntry {
                    rule: outcome.id.clone(),
                    from: violation.from.to_string(),
                    to: violation.to.to_string(),
                })
            })
            .collect();
        Self {
            held_on: held_on.to_owned(),
            entries: entries.into_iter().collect(),
        }
    }

    /// Reads the file.
    ///
    /// # Errors
    /// `CFG-102` if the text is not a valid held state.
    pub fn parse(text: &str) -> Result<Self, CodedError> {
        toml::from_str(text).map_err(|_| CodedError::new(ErrorCode::ConfigParseFailed))
    }

    /// The text of the file.
    ///
    /// # Errors
    /// `CFG-102` if the state cannot be written as TOML.
    pub fn to_text(&self) -> Result<String, CodedError> {
        toml::to_string(self).map_err(|_| CodedError::new(ErrorCode::ConfigParseFailed))
    }

    /// Marks the violations of `outcomes` that this state holds as old, and lists the held
    /// entries that no violation matches any more.
    #[must_use]
    pub fn judge(&self, mut outcomes: Vec<Outcome>) -> Judged {
        let held: BTreeSet<&BaselineEntry> = self.entries.iter().collect();
        let mut seen: BTreeSet<BaselineEntry> = BTreeSet::new();
        for outcome in &mut outcomes {
            for violation in &mut outcome.violations {
                let entry = BaselineEntry {
                    rule: outcome.id.clone(),
                    from: violation.from.to_string(),
                    to: violation.to.to_string(),
                };
                violation.old = held.contains(&entry);
                seen.insert(entry);
            }
        }
        let resolved = self
            .entries
            .iter()
            .filter(|entry| !seen.contains(*entry))
            .cloned()
            .collect();
        Judged { outcomes, resolved }
    }
}
