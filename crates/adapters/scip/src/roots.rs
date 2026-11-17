//! The roots of a namespace symbol (ADR 0061): the files that the index defines as a module.
//!
//! rust-analyzer defines the module of a file (`crate/` for `lib.rs`, `ports/` for `ports/mod.rs`)
//! over the whole file, whatever the file is called. The module symbol of level 1 is called after
//! the file (`lib`, `mod`), so the name rule of the import misses it; the position settles it. One
//! symbol can have several roots: the library and the build script of a package are both `crate/`.
//! A use then belongs to one of them, by a fixed rule.

use std::collections::HashMap;
use std::rc::Rc;

use codeskimr_domain::{FileId, SymbolId};

use crate::consts::{MAX_ROOTS_PER_SYMBOL, PATH_JOIN, ROOT_FOLDER};
use crate::model::SymbolRef;

/// A file the index defines as a module, with the level 1 module symbol of that file.
struct Root<'a> {
    file: FileId,
    path: &'a str,
    module: Rc<[SymbolId]>,
}

/// What the roots say about the use of a symbol.
pub(crate) enum Pick {
    /// The symbol has no root: the other rules decide.
    NoRoots,
    /// The module of the root that the use belongs to.
    Module(Rc<[SymbolId]>),
    /// The symbol has several roots and the rule names none of them.
    Nothing,
}

/// The roots of all symbols of an index, and the name of the file that wins a tie.
pub(crate) struct Roots<'a> {
    by_symbol: HashMap<SymbolRef, Vec<Root<'a>>>,
    library_root: &'a str,
}

/// The folder of a path: everything before its last separator.
fn folder(path: &str) -> &str {
    path.rsplit_once(PATH_JOIN)
        .map_or(ROOT_FOLDER, |(folder, _)| folder)
}

/// The last component of a path.
fn file_name(path: &str) -> &str {
    path.rsplit_once(PATH_JOIN).map_or(path, |(_, name)| name)
}

/// Whether `outer` is `inner` or a folder above it.
fn encloses(outer: &str, inner: &str) -> bool {
    outer == ROOT_FOLDER
        || inner == outer
        || inner
            .strip_prefix(outer)
            .is_some_and(|rest| rest.starts_with(PATH_JOIN))
}

impl<'a> Roots<'a> {
    /// No roots yet; `library_root` is the file name that wins when the folders do not decide
    /// (`[indexer] library_root` of the language pack; empty if the pack names none).
    pub(crate) fn new(library_root: &'a str) -> Self {
        Self {
            by_symbol: HashMap::new(),
            library_root,
        }
    }

    /// Records that `file` (at `path`) is a root of `symbol` with this level 1 module.
    pub(crate) fn add(&mut self, symbol: SymbolRef, file: FileId, path: &'a str, module: SymbolId) {
        let roots = self.by_symbol.entry(symbol).or_default();
        if roots.len() < MAX_ROOTS_PER_SYMBOL && roots.iter().all(|root| root.file != file) {
            roots.push(Root {
                file,
                path,
                module: Rc::from([module].as_slice()),
            });
        }
    }

    /// The module that a use of `symbol` in the file at `path` belongs to. One root is the answer
    /// for every use. For several: (a) the root in the same file, else (b) the one whose folder is
    /// the deepest folder above the use, if there is only one such, else (c) the one root that
    /// bears the name of the library root, else nothing.
    pub(crate) fn pick(&self, symbol: SymbolRef, file: FileId, path: &str) -> Pick {
        let Some(roots) = self.by_symbol.get(&symbol) else {
            return Pick::NoRoots;
        };
        let chosen = match roots.as_slice() {
            [only] => Some(only),
            _ => Self::by_place(roots, file, path).or_else(|| self.library(roots)),
        };
        chosen.map_or(Pick::Nothing, |root| Pick::Module(Rc::clone(&root.module)))
    }

    /// Rules (a) and (b).
    fn by_place<'r>(roots: &'r [Root<'a>], file: FileId, path: &str) -> Option<&'r Root<'a>> {
        if let Some(same) = roots.iter().find(|root| root.file == file) {
            return Some(same);
        }
        let above: Vec<&Root<'a>> = roots
            .iter()
            .filter(|root| encloses(folder(root.path), folder(path)))
            .collect();
        let deepest = above.iter().map(|root| folder(root.path).len()).max()?;
        let mut best = above
            .into_iter()
            .filter(|root| folder(root.path).len() == deepest);
        let first = best.next()?;
        best.next().is_none().then_some(first)
    }

    /// Rule (c): the only root with the file name the pack names.
    fn library<'r>(&self, roots: &'r [Root<'a>]) -> Option<&'r Root<'a>> {
        let mut named = roots
            .iter()
            .filter(|root| file_name(root.path) == self.library_root);
        let first = named.next()?;
        named.next().is_none().then_some(first)
    }
}
