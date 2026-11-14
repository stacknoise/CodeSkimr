//! Names declared in a file and the scopes they are visible in (blueprint 9.2, steps 1 and 3).
//!
//! A local variable or parameter hides a member of the same name, so an occurrence of it is not a
//! reference to a symbol. A declared type tells which type a receiver has.

use std::collections::HashMap;
use std::ops::Range;

/// A declaration as the query found it, before its scope is known.
#[derive(Debug, Clone)]
pub struct RawDeclaration {
    /// The declared name.
    pub name: String,
    /// The type named in the declaration, if the syntax names one.
    pub declared_type: Option<String>,
    /// Byte offset of the name.
    pub at: usize,
    /// Whether it is a local variable or parameter (it hides members) and not a field.
    pub local: bool,
}

/// A name visible in a scope.
#[derive(Debug, Clone)]
struct Declaration {
    raw: RawDeclaration,
    scope: Range<usize>,
}

/// What a lookup of a name found.
#[derive(Debug, Clone, Copy)]
pub struct Declared<'a> {
    /// The declared type, if the syntax names one.
    pub declared_type: Option<&'a str>,
    /// Whether the name is a local variable or parameter.
    pub local: bool,
}

/// All declarations of a file with their scopes.
#[derive(Debug, Default)]
pub struct Scopes {
    declarations: Vec<Declaration>,
}

impl Scopes {
    /// Places every declaration in the smallest scope that contains it. A declaration that the
    /// query found several times, with and without a type, keeps the type.
    #[must_use]
    pub fn new(scopes: &[Range<usize>], raw: Vec<RawDeclaration>) -> Self {
        let mut unique: HashMap<usize, RawDeclaration> = HashMap::new();
        for declaration in raw {
            let kept = unique
                .entry(declaration.at)
                .or_insert_with(|| declaration.clone());
            if kept.declared_type.is_none() {
                kept.declared_type = declaration.declared_type;
            }
        }
        let declarations = unique
            .into_values()
            .filter_map(|raw| {
                let scope = innermost(scopes, raw.at)?;
                Some(Declaration { raw, scope })
            })
            .collect();
        Self { declarations }
    }

    /// The declaration that `name` at byte offset `at` refers to: the innermost scope wins, and a
    /// local variable counts only after its declaration.
    #[must_use]
    pub fn find(&self, name: &str, at: usize) -> Option<Declared<'_>> {
        self.innermost_match(name, at, |found| !found.raw.local || found.raw.at <= at)
    }

    /// Like [`Scopes::find`], but only a field or property: `this.name` never means a local.
    #[must_use]
    pub fn find_member(&self, name: &str, at: usize) -> Option<Declared<'_>> {
        self.innermost_match(name, at, |found| !found.raw.local)
    }

    /// The declaration called `name` whose scope contains `at` and that passes `accept`; the
    /// innermost scope wins, and within it the latest declaration.
    fn innermost_match(
        &self,
        name: &str,
        at: usize,
        accept: impl Fn(&Declaration) -> bool,
    ) -> Option<Declared<'_>> {
        self.declarations
            .iter()
            .filter(|found| found.raw.name == name && found.scope.contains(&at) && accept(found))
            .min_by_key(|found| (found.scope.len(), usize::MAX - found.raw.at))
            .map(|found| Declared {
                declared_type: found.raw.declared_type.as_deref(),
                local: found.raw.local,
            })
    }
}

fn innermost(scopes: &[Range<usize>], at: usize) -> Option<Range<usize>> {
    scopes
        .iter()
        .filter(|scope| scope.contains(&at))
        .min_by_key(|scope| scope.len())
        .cloned()
}
