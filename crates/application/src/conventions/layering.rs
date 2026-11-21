//! `CHK-103` against the stored project: a symbol that uses a layer its own layer may not use.

use std::collections::BTreeMap;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Layer, LayerName, QualifiedName, RelativePath, Span};

use super::model::{Detail, Finding, Location, StoredProject};
use crate::error::CodedError;
use crate::ports::ReferenceSite;

/// Whether the layers can be violated at all: at least one of them limits what it may use.
pub(super) fn can_be_checked(layers: &[Layer]) -> bool {
    layers.iter().any(|layer| layer.may_use.is_some())
}

/// The layer that owns a qualified name: the first one, as in the layer graph of the snapshot.
fn layer_of<'a>(layers: &'a [Layer], name: &QualifiedName) -> Option<&'a Layer> {
    layers.iter().find(|layer| layer.owns(name.as_str()))
}

/// What one finding is made of: the symbol and the layer it reaches into.
type Key = (RelativePath, QualifiedName, String);

/// One finding for every symbol and forbidden layer, with the number of references. Only the
/// stored references of the checked `files` count, so edits that no scan has seen are not
/// reflected.
///
/// # Errors
/// A coded failure of the `STO` group.
pub(super) fn layering(
    stored: &StoredProject<'_>,
    files: &[RelativePath],
) -> Result<Vec<Finding>, CodedError> {
    if !can_be_checked(stored.layers) {
        return Ok(Vec::new());
    }
    let sites = stored.references.references_from(stored.snapshot, files)?;
    let mut counts: BTreeMap<Key, (Span, String, u32)> = BTreeMap::new();
    for site in &sites {
        if let Some((from, to)) = violation(stored.layers, site) {
            let key = (site.path.clone(), site.from.clone(), to.to_string());
            let entry = counts
                .entry(key)
                .or_insert((site.span, from.to_string(), 0));
            entry.2 = entry.2.saturating_add(1);
        }
    }
    Ok(counts
        .into_iter()
        .map(
            |((path, symbol, to_layer), (span, from_layer, count))| Finding {
                code: ErrorCode::LayerViolation,
                location: Location { path, span, symbol },
                detail: Detail::Layering {
                    from_layer,
                    to_layer,
                    count,
                },
            },
        )
        .collect())
}

/// The layer of the symbol and the layer it reaches into, if the reference leaves what the
/// first one allows.
fn violation<'a>(
    layers: &'a [Layer],
    site: &ReferenceSite,
) -> Option<(&'a LayerName, &'a LayerName)> {
    let from = layer_of(layers, &site.from)?;
    let to = layer_of(layers, &site.to)?;
    (from.name != to.name && !from.allows(&to.name)).then_some((&from.name, &to.name))
}
