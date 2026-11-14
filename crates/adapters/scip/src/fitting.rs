//! Finds the one project symbol that a name and a path of an impl symbol can mean. The candidates
//! are indexed once by the end of their modules, so a lookup costs the length of the path and not
//! the number of candidates, whatever an index holds (it is untrusted input).

use std::collections::HashMap;

use crate::consts::{MAX_PATH_DEPTH, MAX_SYMBOLS_PER_KEY};
use crate::impl_symbol::TypeName;
use crate::model::SymbolRef;

/// The key of a lookup: the name of the owner (empty for none), the name, the modules.
type Key = (String, String, Vec<String>);

/// At most two symbols per key: a second one already makes the answer ambiguous.
fn remember(found: &mut HashMap<Key, Vec<SymbolRef>>, key: Key, at: SymbolRef) {
    let symbols = found.entry(key).or_default();
    if symbols.len() < MAX_SYMBOLS_PER_KEY {
        symbols.push(at);
    }
}

/// Candidates that can be found by name and path.
#[derive(Debug, Default)]
pub(crate) struct Fitting {
    /// Under every end of the modules of the symbol (at most `MAX_PATH_DEPTH` of them).
    by_end: HashMap<Key, Vec<SymbolRef>>,
    /// Under all of its modules, if there are not more than `MAX_PATH_DEPTH`.
    by_root: HashMap<Key, Vec<SymbolRef>>,
}

impl Fitting {
    /// Adds a symbol called `name` (inside `owner`, if it has one) that lies in `modules`.
    pub(crate) fn add(&mut self, owner: &str, name: &str, modules: &[String], at: SymbolRef) {
        let key = |path: &[String]| (owner.to_owned(), name.to_owned(), path.to_vec());
        if modules.len() <= MAX_PATH_DEPTH {
            remember(&mut self.by_root, key(modules), at);
        }
        let tail = modules
            .get(modules.len().saturating_sub(MAX_PATH_DEPTH)..)
            .unwrap_or_default();
        for from in 0..=tail.len() {
            remember(
                &mut self.by_end,
                key(tail.get(from..).unwrap_or_default()),
                at,
            );
        }
    }

    /// The one symbol of that owner and name that fits `ty`: a path from the crate root must be
    /// exactly its modules, any other path must be their end, no path fits every module.
    pub(crate) fn unique(&self, owner: &str, ty: &TypeName) -> Option<SymbolRef> {
        if ty.path.len() > MAX_PATH_DEPTH {
            return None;
        }
        let index = if ty.rooted {
            &self.by_root
        } else {
            &self.by_end
        };
        let key = (owner.to_owned(), ty.name.clone(), ty.path.clone());
        match index.get(&key)?.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }
}
