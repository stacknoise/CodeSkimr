//! Looks at every certain reference once and sorts it into the rules it breaks (ADR 0104).

use std::collections::BTreeMap;

use codeskimr_domain::{QualifiedName, is_within};

use super::consts::{ID_ARROW, ID_FORBID, ID_PUBLIC_API, ID_SEPARATOR};
use super::cycles::cycles_outcome;
use super::model::{Outcome, Place, RuleKind, Violation};
use crate::config::{ForbidRule, PublicApiRule, Rules};
use crate::error::CodedError;
use crate::ports::{ReferencePlace, ReferencePlaces};
use codeskimr_domain::SnapshotId;

/// What a rule has found so far: the places of each pair of symbols.
type Found = BTreeMap<(QualifiedName, QualifiedName), Vec<Place>>;

/// A rule of the project file, ready to look at a reference.
enum Rule<'a> {
    Forbid(&'a ForbidRule),
    PublicApi(&'a PublicApiRule),
}

impl Rule<'_> {
    fn id(&self) -> String {
        match self {
            Self::Forbid(rule) => format!(
                "{ID_FORBID}{ID_SEPARATOR}{}{ID_ARROW}{}",
                rule.from, rule.to
            ),
            Self::PublicApi(rule) => format!("{ID_PUBLIC_API}{ID_SEPARATOR}{}", rule.module),
        }
    }

    const fn kind(&self) -> RuleKind {
        match self {
            Self::Forbid(_) => RuleKind::Forbid,
            Self::PublicApi(_) => RuleKind::PublicApi,
        }
    }

    /// Whether the reference breaks the rule.
    fn breaks(&self, from: &str, to: &str) -> bool {
        match self {
            Self::Forbid(rule) => is_within(&rule.from, from) && is_within(&rule.to, to),
            Self::PublicApi(rule) => {
                !is_within(&rule.module, from)
                    && rule.internal.iter().any(|inside| is_within(inside, to))
            }
        }
    }
}

/// The rules of the project in the order of the project file: `forbid`, then `public_api`.
fn rules_of(rules: &Rules) -> Vec<Rule<'_>> {
    let forbid = rules.forbid.iter().map(Rule::Forbid);
    forbid
        .chain(rules.public_api.iter().map(Rule::PublicApi))
        .collect()
}

/// Evaluates the rules of `rules` against the certain references of `snapshot`. A rule without a
/// violation is in the answer with an empty list, so the screen can say "kept".
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn evaluate(
    rules: &Rules,
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
) -> Result<Vec<Outcome>, CodedError> {
    let cycles = cycles_outcome(rules, references, snapshot)?;
    let rules = rules_of(rules);
    let mut found: Vec<Found> = rules.iter().map(|_| Found::new()).collect();
    references.for_each_reference(snapshot, &mut |reference: ReferencePlace| {
        for (rule, hits) in rules.iter().zip(&mut found) {
            if reference.from != reference.to
                && rule.breaks(reference.from.as_str(), reference.to.as_str())
            {
                hits.entry((reference.from.clone(), reference.to.clone()))
                    .or_default()
                    .push(Place {
                        path: reference.path.clone(),
                        span: reference.span,
                    });
            }
        }
    })?;
    let judged = rules.iter().zip(found).map(|(rule, hits)| Outcome {
        id: rule.id(),
        kind: rule.kind(),
        violations: hits
            .into_iter()
            .map(|((from, to), places)| Violation {
                from,
                to,
                places,
                old: false,
                ring: None,
            })
            .collect(),
    });
    Ok(cycles.into_iter().chain(judged).collect())
}
