//! The capture vocabulary of language pack queries, resolved once when a pack loads.

use std::str::FromStr;

use codeskimr_domain::{ReferenceKind, SymbolKind};
use tree_sitter::Query;

use crate::consts::{
    COGNITIVE_CAPTURE, COGNITIVE_FLAT_CAPTURE, COGNITIVE_SEQUENCE_CAPTURE, COMMENT_CAPTURE,
    CONTAINER_CAPTURE, CONTINUATION_CAPTURE, DECISION_CAPTURE, DECLARED_TYPE_CAPTURE, DOC_CAPTURE,
    FIELD_CAPTURE, HELPER_CAPTURE_PREFIX, LIFT_CAPTURE, LITERAL_CAPTURE, LITERAL_EXEMPT_CAPTURE,
    LOCAL_CAPTURE, MODIFIER_CAPTURE, NAME_CAPTURE, NESTING_CAPTURE, OPERAND_CAPTURE,
    OPERATOR_CAPTURE, OWNER_CAPTURE, PARAMETER_CAPTURE, RECEIVER_BASE_CAPTURE, RECEIVER_CAPTURE,
    RECEIVER_MEMBER_CAPTURE, RECEIVER_OTHER_CAPTURE, RECEIVER_OWN_CAPTURE, RECEIVER_TYPE_CAPTURE,
    REFERENCE_CAPTURE_PREFIX, SCOPE_CAPTURE, SCOPE_END_CAPTURE, SIGNATURE_CAPTURE, STATIC_CAPTURE,
    SYMBOL_CAPTURE_PREFIX, VISIBILITY_CAPTURE,
};

/// What the syntax says about the receiver in front of a member name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverRole {
    /// A name: a variable, a field or a type.
    Name,
    /// The type of a creation expression.
    Type,
    /// A field or property named behind `this.`.
    Member,
    /// The enclosing object itself.
    Own,
    /// The base of the enclosing object.
    Base,
    /// Anything else; nothing is known about it.
    Other,
}

/// What a capture of a query stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    /// A whole declaration of this kind.
    Symbol(SymbolKind),
    /// An occurrence of this kind.
    Reference(ReferenceKind),
    /// Simple name of a declaration or target of a reference.
    Name,
    /// Visibility keyword; may occur several times, such as `protected` and `internal`.
    Visibility,
    /// Any modifier keyword; the pack decides which ones set visibility or static.
    Modifier,
    /// Marker for static declarations.
    Static,
    /// Documentation comment.
    Doc,
    /// Signature text.
    Signature,
    /// Node whose end is the end of the symbol's scope.
    ScopeEnd,
    /// The receiver in front of the captured name.
    Receiver(ReceiverRole),
    /// The name of a local variable or parameter.
    Local,
    /// The name of a field or property.
    Field,
    /// The type named in a declaration.
    DeclaredType,
    /// A node that opens a scope.
    Scope,
    /// A block without a symbol of its own whose members belong to the type named by `@name`.
    Container,
    /// The type a declaration written outside the type belongs to (ADR 0042).
    Owner,
    /// Marker: the declaration belongs to the parent of the symbol it is written in.
    Lift,
    /// A node that only a predicate looks at (`_keyword` in `(#match? @_keyword ...)`); it has no
    /// meaning for the adapter (ADR 0113).
    Helper,
}

impl Capture {
    /// Interprets a capture name; `None` if it is not part of the vocabulary.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        if name.starts_with(HELPER_CAPTURE_PREFIX) {
            return Some(Self::Helper);
        }
        if let Some(label) = name.strip_prefix(SYMBOL_CAPTURE_PREFIX) {
            return SymbolKind::from_str(label).ok().map(Self::Symbol);
        }
        if let Some(label) = name.strip_prefix(REFERENCE_CAPTURE_PREFIX) {
            return ReferenceKind::from_str(label).ok().map(Self::Reference);
        }
        match name {
            NAME_CAPTURE => Some(Self::Name),
            VISIBILITY_CAPTURE => Some(Self::Visibility),
            MODIFIER_CAPTURE => Some(Self::Modifier),
            STATIC_CAPTURE => Some(Self::Static),
            DOC_CAPTURE => Some(Self::Doc),
            SIGNATURE_CAPTURE => Some(Self::Signature),
            SCOPE_END_CAPTURE => Some(Self::ScopeEnd),
            RECEIVER_CAPTURE => Some(Self::Receiver(ReceiverRole::Name)),
            RECEIVER_TYPE_CAPTURE => Some(Self::Receiver(ReceiverRole::Type)),
            RECEIVER_MEMBER_CAPTURE => Some(Self::Receiver(ReceiverRole::Member)),
            RECEIVER_OWN_CAPTURE => Some(Self::Receiver(ReceiverRole::Own)),
            RECEIVER_BASE_CAPTURE => Some(Self::Receiver(ReceiverRole::Base)),
            RECEIVER_OTHER_CAPTURE => Some(Self::Receiver(ReceiverRole::Other)),
            LOCAL_CAPTURE => Some(Self::Local),
            FIELD_CAPTURE => Some(Self::Field),
            DECLARED_TYPE_CAPTURE => Some(Self::DeclaredType),
            SCOPE_CAPTURE => Some(Self::Scope),
            CONTAINER_CAPTURE => Some(Self::Container),
            OWNER_CAPTURE => Some(Self::Owner),
            LIFT_CAPTURE => Some(Self::Lift),
            _ => None,
        }
    }
}

/// A query together with the meaning of each of its captures.
pub struct Vocabulary {
    /// The compiled query.
    pub query: Query,
    roles: Vec<Capture>,
}

impl Vocabulary {
    /// Resolves every capture name of `query`.
    ///
    /// # Errors
    /// The offending capture name if it is not part of the vocabulary.
    pub fn new(query: Query) -> Result<Self, String> {
        let roles = query
            .capture_names()
            .iter()
            .map(|name| Capture::parse(name).ok_or_else(|| (*name).to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { query, roles })
    }

    /// Meaning of the capture with this index.
    #[must_use]
    pub fn role(&self, index: u32) -> Option<Capture> {
        self.roles.get(usize::try_from(index).ok()?).copied()
    }
}

/// What a capture of `metrics.scm` stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricRole {
    /// A comment; the line kinds come from `comment_nodes`, so the capture is accepted but unused.
    Comment,
    /// A decision point.
    Decision,
    /// An operator token.
    Operator,
    /// An operand.
    Operand,
    /// A structure that adds to the cognitive complexity, more the deeper it is nested.
    Cognitive,
    /// A structure that adds one to the cognitive complexity at any depth.
    CognitiveFlat,
    /// The operator of a boolean operation; a run of the same one counts once.
    CognitiveSequence,
    /// A node that nests what is inside it.
    Nesting,
    /// A structure that continues the chain of the one around it, such as `else if`.
    Continuation,
    /// A parameter of a callable declaration.
    Parameter,
    /// A literal the rule of no literals reports.
    Literal,
    /// A node whose literal is an exception: it is not reported although `@literal` matches it.
    LiteralExempt,
    /// A node that only a predicate looks at (`_word`); it counts for nothing (ADR 0113).
    Helper,
}

/// The compiled `metrics.scm` of a pack.
pub struct MetricsQuery {
    pub(crate) query: Query,
    pub(crate) roles: Vec<MetricRole>,
}

impl MetricsQuery {
    /// Whether the pack counts the cognitive complexity (it captures `@cognitive`).
    #[must_use]
    pub fn counts_cognitive(&self) -> bool {
        self.roles.contains(&MetricRole::Cognitive)
    }

    /// Whether the pack reports literals (it captures `@literal`, ADR 0058).
    #[must_use]
    pub fn counts_literals(&self) -> bool {
        self.roles.contains(&MetricRole::Literal)
    }

    /// Whether the pack counts parameters (it captures `@parameter`).
    #[must_use]
    pub fn counts_parameters(&self) -> bool {
        self.roles.contains(&MetricRole::Parameter)
    }
}

impl MetricsQuery {
    /// Resolves every capture name of `query`.
    ///
    /// # Errors
    /// The offending capture name if it is not one of ADR 0015 and ADR 0048.
    // codeskimr: allow CHK-102 maps the capture names of the metrics query into its roles
    pub fn new(query: Query) -> Result<Self, String> {
        let roles = query
            .capture_names()
            .iter()
            .map(|name| role_of(name).ok_or_else(|| (*name).to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { query, roles })
    }
}

fn role_of(name: &str) -> Option<MetricRole> {
    if name.starts_with(HELPER_CAPTURE_PREFIX) {
        return Some(MetricRole::Helper);
    }
    match name {
        COMMENT_CAPTURE => Some(MetricRole::Comment),
        DECISION_CAPTURE => Some(MetricRole::Decision),
        OPERATOR_CAPTURE => Some(MetricRole::Operator),
        OPERAND_CAPTURE => Some(MetricRole::Operand),
        COGNITIVE_CAPTURE => Some(MetricRole::Cognitive),
        COGNITIVE_FLAT_CAPTURE => Some(MetricRole::CognitiveFlat),
        COGNITIVE_SEQUENCE_CAPTURE => Some(MetricRole::CognitiveSequence),
        NESTING_CAPTURE => Some(MetricRole::Nesting),
        CONTINUATION_CAPTURE => Some(MetricRole::Continuation),
        PARAMETER_CAPTURE => Some(MetricRole::Parameter),
        LITERAL_CAPTURE => Some(MetricRole::Literal),
        LITERAL_EXEMPT_CAPTURE => Some(MetricRole::LiteralExempt),
        _ => None,
    }
}
