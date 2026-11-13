//! Which symbol kinds a kind of occurrence can point to, best candidates first.

use codeskimr_domain::{ReferenceKind, SymbolKind};

/// Kinds that are types.
pub const TYPES: &[SymbolKind] = &[
    SymbolKind::Class,
    SymbolKind::Interface,
    SymbolKind::Trait,
    SymbolKind::Record,
    SymbolKind::Struct,
    SymbolKind::Enum,
    SymbolKind::Union,
    SymbolKind::TypeAlias,
];
/// Kinds that can be called.
const CALLABLES: &[SymbolKind] = &[
    SymbolKind::Method,
    SymbolKind::Function,
    SymbolKind::Constructor,
];
/// Kinds that hold a value.
const VALUES: &[SymbolKind] = &[
    SymbolKind::Property,
    SymbolKind::Field,
    SymbolKind::Constant,
    SymbolKind::EnumMember,
    SymbolKind::Event,
];
/// Kinds of members that a subtype can replace; a call through the base may reach the subtype.
pub const OVERRIDABLE: &[SymbolKind] =
    &[SymbolKind::Method, SymbolKind::Property, SymbolKind::Event];
/// Kinds that an import names.
const NAMESPACES: &[SymbolKind] = &[SymbolKind::Namespace, SymbolKind::Module];
/// Kinds of types that only declare what implementers provide.
const CONTRACTS: &[SymbolKind] = &[SymbolKind::Interface, SymbolKind::Trait];

const CALL: &[&[SymbolKind]] = &[CALLABLES, VALUES];
const READ: &[&[SymbolKind]] = &[VALUES, TYPES, CALLABLES];
const WRITE: &[&[SymbolKind]] = &[VALUES];
const TYPE_USE: &[&[SymbolKind]] = &[TYPES];
const OVERRIDE: &[&[SymbolKind]] = &[OVERRIDABLE];
const IMPORT: &[&[SymbolKind]] = &[NAMESPACES];
const TEST: &[&[SymbolKind]] = &[CALLABLES, VALUES, TYPES];

/// The kinds an occurrence of `kind` can point to, in order of preference: a lookup tries the
/// first group completely before it looks at the next.
pub const fn candidates(kind: ReferenceKind) -> &'static [&'static [SymbolKind]] {
    match kind {
        ReferenceKind::Call => CALL,
        ReferenceKind::Read => READ,
        ReferenceKind::Write => WRITE,
        ReferenceKind::TypeUse | ReferenceKind::Inherit | ReferenceKind::Implement => TYPE_USE,
        ReferenceKind::Override => OVERRIDE,
        ReferenceKind::Import => IMPORT,
        ReferenceKind::Test => TEST,
    }
}

/// Whether the kind is a type.
pub fn is_type(kind: SymbolKind) -> bool {
    TYPES.contains(&kind)
}

/// Whether the kind only declares members that implementers provide.
pub fn is_contract(kind: SymbolKind) -> bool {
    CONTRACTS.contains(&kind)
}

/// Whether a subtype can replace members of this kind.
pub fn is_overridable(kind: SymbolKind) -> bool {
    OVERRIDABLE.contains(&kind)
}

/// The kind a reference is reported as once its target is known: a name in a value position that
/// turns out to be a type is a type use, and a base type that is an interface is implemented.
pub fn reported(kind: ReferenceKind, target: SymbolKind) -> ReferenceKind {
    match kind {
        ReferenceKind::Read if is_type(target) => ReferenceKind::TypeUse,
        ReferenceKind::Inherit if is_contract(target) => ReferenceKind::Implement,
        other => other,
    }
}
