//! What the evaluation of the architecture rules answers.

use codeskimr_domain::{QualifiedName, RelativePath, Span};

use super::consts::ID_MODULES_JOIN;

/// The kind of a rule (ADR 0105 to 0107; the layers are the rule of ADR 0054).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    /// `rules.cycles = "deny"`: modules must not depend on each other in a ring.
    Cycles,
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

/// The ring of modules behind a cycle violation (ADR 0105).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ring {
    /// Every module of the ring, sorted: its identity in the held state.
    pub modules: Vec<QualifiedName>,
    /// A way round the ring that starts with the weakest edge and ends at its start (`A, B, A`).
    pub chain: Vec<QualifiedName>,
    /// How many references the weakest edge has.
    pub weakest_count: u32,
    /// Over how many edges the ring is resolved when the weakest ones are removed.
    pub resolution_edges: usize,
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
    /// For a cycle, the ring; `from` and `to` are then the ends of its weakest edge and `places`
    /// its references.
    pub ring: Option<Ring>,
}

impl Violation {
    /// What the held state writes as `from` and `to` of this violation: the two symbols, or for a
    /// cycle the sorted modules of the ring and nothing.
    #[must_use]
    pub fn identity(&self) -> (String, String) {
        match &self.ring {
            Some(ring) => (
                ring.modules
                    .iter()
                    .map(QualifiedName::as_str)
                    .collect::<Vec<_>>()
                    .join(ID_MODULES_JOIN),
                String::new(),
            ),
            None => (self.from.to_string(), self.to.to_string()),
        }
    }
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
