//! The symbols behind the names of the violations of the screen "Rules", so that a click on a
//! name can open the symbol in the explorer.

use std::collections::HashMap;

use codeskimr_application::ports::{CodeIndex, SymbolKey};
use codeskimr_codes::Coded;
use codeskimr_domain::{QualifiedName, SnapshotId};

use super::rules::{RuleOut, SymbolRef};

/// Looks one name up in the snapshot. A name that no symbol carries (a namespace that only the
/// layers know, a symbol the scan did not see) or a failed read gives `None`: the name is then
/// shown as plain text.
pub(super) fn resolve(
    index: &dyn CodeIndex,
    snapshot: SnapshotId,
    name: &str,
) -> Option<SymbolRef> {
    let key = SymbolKey::Qualified(QualifiedName::new(name).ok()?);
    let found = index
        .symbol(snapshot, &key)
        .inspect_err(|error| tracing::warn!(code = error.code().id()))
        .ok()
        .flatten()?;
    Some(SymbolRef {
        id: found.symbol.id.get(),
        kind: found.symbol.kind.label(),
    })
}

/// Fills `from_symbol` and `to_symbol` of every violation. Each name is looked up once.
pub(super) fn attach_symbols(index: &dyn CodeIndex, snapshot: SnapshotId, rules: &mut [RuleOut]) {
    let mut known: HashMap<String, Option<SymbolRef>> = HashMap::new();
    let mut lookup = |name: &str| {
        known
            .entry(name.to_owned())
            .or_insert_with(|| resolve(index, snapshot, name))
            .clone()
    };
    for violation in rules.iter_mut().flat_map(|rule| rule.violations.iter_mut()) {
        violation.from_symbol = lookup(&violation.from);
        violation.to_symbol = lookup(&violation.to);
    }
}
