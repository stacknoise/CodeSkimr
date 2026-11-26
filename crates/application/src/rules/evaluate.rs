//! Looks at every certain reference once and sorts it into the rules it breaks (ADR 0104).

use std::collections::BTreeMap;

use codeskimr_domain::{Layer, QualifiedName, is_within};

use super::consts::{ID_ARROW, ID_FORBID, ID_LAYERS, ID_PUBLIC_API, ID_SEPARATOR};
use super::model::{Outcome, Place, RuleKind, Violation};
use crate::config::{ForbidRule, PublicApiRule, Rules};
use crate::error::CodedError;
use crate::ports::{ReferencePlace, ReferencePlaces};
use codeskimr_domain::SnapshotId;

/// What a rule has found so far: the places of each pair of symbols.
type Found = BTreeMap<(QualifiedName, QualifiedName), Vec<Place>>;

/// A rule of the project file, ready to look at a reference.
enum Rule<'a> {
    Layers(&'a [Layer]),
    Forbid(&'a ForbidRule),
    PublicApi(&'a PublicApiRule),
}

impl Rule<'_> {
    fn id(&self) -> String {
        match self {
            Self::Layers(_) => ID_LAYERS.to_owned(),
            Self::Forbid(rule) => format!(
                "{ID_FORBID}{ID_SEPARATOR}{}{ID_ARROW}{}",
                rule.from, rule.to
            ),
            Self::PublicApi(rule) => format!("{ID_PUBLIC_API}{ID_SEPARATOR}{}", rule.module),
        }
    }

    const fn kind(&self) -> RuleKind {
        match self {
            Self::Layers(_) => RuleKind::Layers,
            Self::Forbid(_) => RuleKind::Forbid,
            Self::PublicApi(_) => RuleKind::PublicApi,
        }
    }

    /// Whether the reference breaks the rule.
    fn breaks(&self, from: &str, to: &str) -> bool {
        match self {
            Self::Layers(layers) => crosses(layers, from, to),
            Self::Forbid(rule) => is_within(&rule.from, from) && is_within(&rule.to, to),
            Self::PublicApi(rule) => {
                !is_within(&rule.module, from)
                    && rule.internal.iter().any(|inside| is_within(inside, to))
            }
        }
    }
}

/// Whether a reference leaves what the layer of its source allows (ADR 0082): both ends belong to
/// a layer, the layers differ and the first does not list the second. The first layer that owns a
/// name wins, as in the layer graph.
fn crosses(layers: &[Layer], from: &str, to: &str) -> bool {
    let owner = |name: &str| layers.iter().find(|layer| layer.owns(name));
    match (owner(from), owner(to)) {
        (Some(source), Some(target)) => source.name != target.name && !source.allows(&target.name),
        _ => false,
    }
}

/// The rules in the order the screen shows them: the layers (only if one of them limits what it
/// may use, otherwise nothing can be violated), then `forbid` and `public_api` of the project file.
fn rules_of<'a>(rules: &'a Rules, layers: &'a [Layer]) -> Vec<Rule<'a>> {
    let restricted = layers.iter().any(|layer| layer.may_use.is_some());
    let layered = restricted.then_some(Rule::Layers(layers));
    layered
        .into_iter()
        .chain(rules.forbid.iter().map(Rule::Forbid))
        .chain(rules.public_api.iter().map(Rule::PublicApi))
        .collect()
}

/// Evaluates the layers and the rules of `rules` against the certain references of `snapshot`. A
/// rule without a violation is in the answer with an empty list, so the screen can say "kept".
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn evaluate(
    rules: &Rules,
    layers: &[Layer],
    references: &dyn ReferencePlaces,
    snapshot: SnapshotId,
) -> Result<Vec<Outcome>, CodedError> {
    let rules = rules_of(rules, layers);
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
    Ok(rules
        .iter()
        .zip(found)
        .map(|(rule, hits)| Outcome {
            id: rule.id(),
            kind: rule.kind(),
            violations: hits
                .into_iter()
                .map(|((from, to), places)| Violation {
                    from,
                    to,
                    places,
                    old: false,
                })
                .collect(),
        })
        .collect())
}
