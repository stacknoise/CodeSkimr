//! What the evaluation of the architecture rules answers.

use codeskimr_domain::{QualifiedName, RelativePath, Span};

/// The kind of a rule (ADR 0105 to 0107; the layers are the rule of ADR 0054).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// `[[rules.forbid]]`: module A must not use module B.
    Forbid,
    /// `[[rules.public_api]]`: the inside of a module is closed to others.
    PublicApi,
}

/// One place where the referring symbol reaches the forbidden one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// File of the reference.
    pub path: RelativePath,
    /// Range of the reference, zero-based.
    pub span: Span,
}

/// One violation: the symbol that refers and the one it must not reach, with every place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The referring symbol.
    pub from: QualifiedName,
    /// The symbol it must not reach.
    pub to: QualifiedName,
    /// Every place of the reference, in the order the store gave them.
    pub places: Vec<Place>,
    /// Whether the held state (ADR 0104) already has this violation: an old burden.
    pub old: bool,
}

/// What one rule found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The id of the rule, stable across scans: the kind and the names it is made of. It is the
    /// `rule` of an entry in the held state.
    pub id: String,
    /// The kind of the rule.
    pub kind: RuleKind,
    /// Its violations, ordered by referring and referred symbol.
    pub violations: Vec<Violation>,
}

impl Outcome {
    /// How many violations are not held as old burdens.
    #[must_use]
    pub fn new_violations(&self) -> usize {
        self.violations
            .iter()
            .filter(|violation| !violation.old)
            .count()
    }
}
