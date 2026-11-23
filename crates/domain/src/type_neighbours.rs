//! The neighbours of one type (blueprint 15, screen 3, "type focus"): the types that use it and
//! the types it uses. A reference counts for the type that owns each of its ends: a type owns
//! itself and every member inside it, found by the parent chain (so each part of a partial class
//! keeps its own members, ADR 0036) and, where a symbol has no type among its parents, by the
//! qualified name; a symbol outside every type belongs to none and is not counted. References
//! inside the type are left out.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};

use crate::namespace_graph::enclosing;
use crate::qualified_name::QualifiedName;
use crate::symbol::{SymbolId, SymbolKind};

/// One neighbouring type and the number of references between it and the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeLink {
    /// The neighbouring type.
    pub symbol: SymbolId,
    /// Number of references behind the link.
    pub count: u32,
}

/// The types around one type, the most referenced first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeNeighbours {
    /// Types that use the focus (their references point into it).
    pub users: Vec<TypeLink>,
    /// Types the focus uses (its references point into them).
    pub used: Vec<TypeLink>,
}

/// A symbol of a snapshot: id, kind, qualified name and enclosing symbol.
pub type TypeSymbol<'a> = (SymbolId, SymbolKind, &'a QualifiedName, Option<SymbolId>);

/// The nearest type among the parents of `start`, if the chain reaches one.
fn type_above(
    start: Option<SymbolId>,
    parents: &HashMap<SymbolId, (bool, Option<SymbolId>)>,
) -> Option<SymbolId> {
    let mut at = start;
    for _ in 0..parents.len() {
        let id = at?;
        let (is_type, parent) = *parents.get(&id)?;
        if is_type {
            return Some(id);
        }
        at = parent;
    }
    None
}

/// The type that owns every symbol that has one.
fn owners(symbols: &[TypeSymbol<'_>]) -> HashMap<SymbolId, SymbolId> {
    let types: HashMap<&str, SymbolId> = symbols
        .iter()
        .filter(|(_, kind, _, _)| kind.is_type())
        .map(|(id, _, name, _)| (name.as_str(), *id))
        .collect();
    let parents: HashMap<SymbolId, (bool, Option<SymbolId>)> = symbols
        .iter()
        .map(|(id, kind, _, parent)| (*id, (kind.is_type(), *parent)))
        .collect();
    symbols
        .iter()
        .filter_map(|(id, kind, name, parent)| {
            if kind.is_type() {
                return Some((*id, *id));
            }
            if let Some(owner) = type_above(*parent, &parents) {
                return Some((*id, owner));
            }
            let owner = enclosing(&types, name.as_str())?;
            Some((*id, *types.get(owner)?))
        })
        .collect()
}

fn ranked(counts: BTreeMap<SymbolId, u32>) -> Vec<TypeLink> {
    let mut links: Vec<TypeLink> = counts
        .into_iter()
        .map(|(symbol, count)| TypeLink { symbol, count })
        .collect();
    links.sort_by_key(|link| (Reverse(link.count), link.symbol));
    links
}

/// The symbols a type owns: itself and every member inside it, by the rule of the module. `None`
/// if `focus` is not a type of the symbols. The references that touch these symbols are all that
/// `type_neighbours` needs of the references of a snapshot.
#[must_use]
pub fn type_members<'a>(
    focus: SymbolId,
    symbols: impl IntoIterator<Item = TypeSymbol<'a>>,
) -> Option<Vec<SymbolId>> {
    let symbols: Vec<_> = symbols.into_iter().collect();
    let owner = owners(&symbols);
    if owner.get(&focus) != Some(&focus) {
        return None;
    }
    let mut members: Vec<SymbolId> = owner
        .iter()
        .filter(|(_, owned_by)| **owned_by == focus)
        .map(|(id, _)| *id)
        .collect();
    members.sort_unstable();
    Some(members)
}

/// The neighbours of `focus`, from the symbols (id, kind, qualified name, parent) and the
/// references (from, to) of a snapshot. `None` if `focus` is not a type of the snapshot.
#[must_use]
pub fn type_neighbours<'a>(
    focus: SymbolId,
    symbols: impl IntoIterator<Item = TypeSymbol<'a>>,
    references: impl IntoIterator<Item = (SymbolId, SymbolId)>,
) -> Option<TypeNeighbours> {
    let symbols: Vec<_> = symbols.into_iter().collect();
    let owner = owners(&symbols);
    if owner.get(&focus) != Some(&focus) {
        return None;
    }
    let mut users: BTreeMap<SymbolId, u32> = BTreeMap::new();
    let mut used: BTreeMap<SymbolId, u32> = BTreeMap::new();
    for (from, to) in references {
        let (Some(&a), Some(&b)) = (owner.get(&from), owner.get(&to)) else {
            continue;
        };
        if a == b {
            continue;
        }
        if b == focus {
            *users.entry(a).or_default() += 1;
        } else if a == focus {
            *used.entry(b).or_default() += 1;
        }
    }
    Some(TypeNeighbours {
        users: ranked(users),
        used: ranked(used),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> QualifiedName {
        QualifiedName::new(text).unwrap()
    }

    fn id(value: u64) -> SymbolId {
        SymbolId::new(value)
    }

    fn qualified_names(names: &[(u64, SymbolKind, &str)]) -> Vec<QualifiedName> {
        names.iter().map(|(_, _, text)| name(text)).collect()
    }

    fn triples<'a>(
        names: &'a [(u64, SymbolKind, &str)],
        qualified: &'a [QualifiedName],
    ) -> impl Iterator<Item = TypeSymbol<'a>> {
        names
            .iter()
            .zip(qualified)
            .map(|((value, kind, _), qualified)| (id(*value), *kind, qualified, None))
    }

    #[test]
    fn references_of_members_count_for_their_types_and_inner_ones_are_left_out() {
        let names = [
            (1, SymbolKind::Namespace, "App"),
            (2, SymbolKind::Class, "App.Cart"),
            (3, SymbolKind::Method, "App.Cart.Total"),
            (4, SymbolKind::Class, "App.View"),
            (5, SymbolKind::Method, "App.View.Draw"),
            (6, SymbolKind::Class, "App.Price"),
            (7, SymbolKind::Method, "App.Price.Value"),
            (8, SymbolKind::Method, "App.Cart.Clear"),
        ];
        let qualified = qualified_names(&names);
        let symbols = triples(&names, &qualified);
        let calls = [
            (id(5), id(3)),
            (id(5), id(2)),
            (id(3), id(7)),
            (id(3), id(7)),
            (id(3), id(8)),
            (id(1), id(3)),
        ];
        let found = type_neighbours(id(2), symbols, calls).unwrap();
        assert_eq!(
            found.users,
            [TypeLink {
                symbol: id(4),
                count: 2
            }]
        );
        assert_eq!(
            found.used,
            [TypeLink {
                symbol: id(6),
                count: 2
            }]
        );
    }

    #[test]
    fn the_members_of_a_type_are_the_type_and_what_it_owns() {
        let names = [
            (1, SymbolKind::Namespace, "App"),
            (2, SymbolKind::Class, "App.Cart"),
            (3, SymbolKind::Method, "App.Cart.Total"),
            (4, SymbolKind::Class, "App.View"),
            (5, SymbolKind::Method, "App.View.Draw"),
        ];
        let qualified = qualified_names(&names);
        let members = type_members(id(2), triples(&names, &qualified));
        assert_eq!(members, Some(vec![id(2), id(3)]));
        assert_eq!(type_members(id(1), triples(&names, &qualified)), None);
    }

    #[test]
    fn something_that_is_not_a_type_has_no_neighbours() {
        let app = name("App");
        let symbols = [(id(1), SymbolKind::Namespace, &app, None)];
        assert_eq!(type_neighbours(id(1), symbols, []), None);
        assert_eq!(type_neighbours(id(9), [], []), None);
    }

    #[test]
    fn the_most_referenced_neighbour_comes_first() {
        let names = [
            (1, SymbolKind::Class, "A"),
            (2, SymbolKind::Class, "B"),
            (3, SymbolKind::Class, "C"),
        ];
        let qualified = qualified_names(&names);
        let symbols = triples(&names, &qualified);
        let calls = [(id(1), id(2)), (id(1), id(3)), (id(1), id(3))];
        let found = type_neighbours(id(1), symbols, calls).unwrap();
        let order: Vec<u64> = found.used.iter().map(|link| link.symbol.get()).collect();
        assert_eq!(order, [3, 2]);
    }

    #[test]
    fn the_parts_of_a_partial_class_keep_their_own_members() {
        let part = name("App.Cart");
        let second_part = name("App.Cart");
        let total = name("App.Cart.Total");
        let clear = name("App.Cart.Clear");
        let view = name("App.View");
        let draw = name("App.View.Draw");
        let symbols = [
            (id(1), SymbolKind::Class, &part, None),
            (id(2), SymbolKind::Class, &second_part, None),
            (id(3), SymbolKind::Method, &total, Some(id(1))),
            (id(4), SymbolKind::Method, &clear, Some(id(2))),
            (id(5), SymbolKind::Class, &view, None),
            (id(6), SymbolKind::Method, &draw, Some(id(5))),
        ];
        let calls = [(id(6), id(3)), (id(6), id(4)), (id(6), id(4))];
        let first = type_neighbours(id(1), symbols, calls).unwrap();
        assert_eq!(
            first.users,
            [TypeLink {
                symbol: id(5),
                count: 1
            }]
        );
        let second = type_neighbours(id(2), symbols, calls).unwrap();
        assert_eq!(
            second.users,
            [TypeLink {
                symbol: id(5),
                count: 2
            }]
        );
    }
}
