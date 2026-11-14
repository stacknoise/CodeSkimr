//! Extraction of unresolved name occurrences with the reference query of a language pack.
//!
//! The query also reports what the resolver cannot see because it never gets a syntax tree
//! (ADR 0013): the receiver in front of a member name, and the names declared in the file with
//! their scopes. A name that a local variable or parameter hides is not a reference.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use codeskimr_application::ports::{PendingReference, Receiver};
use codeskimr_domain::{QualifiedName, ReferenceKind};
use tree_sitter::{Node, QueryCursor, QueryMatch, StreamingIterator};

use crate::captures::{Capture, ReceiverRole, Vocabulary};
use crate::error::ParsingError;
use crate::location::{span_of, text_of};
use crate::scopes::{Declared, RawDeclaration, Scopes};
use crate::symbols::{SymbolContext, SymbolTable};

/// What the syntax says about the receiver of a member name, before locals are looked at.
#[derive(Debug, Clone)]
enum Seen {
    /// A name at this byte offset: a variable, a field or a type.
    Name(String, usize),
    /// The type of a creation expression.
    Type(String),
    /// A field or property named behind `this.`, at this byte offset.
    Member(String, usize),
    /// The enclosing object itself.
    Own,
    /// The base of the enclosing object.
    Base,
    /// Anything else.
    Other,
}

/// An occurrence as the query found it.
struct Occurrence<'t> {
    kind: ReferenceKind,
    name: Node<'t>,
}

/// Everything the query reports about a file, in the order it was found.
#[derive(Default)]
struct Facts<'t> {
    occurrences: Vec<Occurrence<'t>>,
    /// Receivers by the byte range of the member name behind them.
    receivers: HashMap<Range<usize>, Seen>,
    declarations: Vec<RawDeclaration>,
    scopes: Vec<Range<usize>>,
}

/// The roles that the captures of one match play.
#[derive(Default)]
struct Roles<'t> {
    kind: Option<ReferenceKind>,
    occurrence: Option<Node<'t>>,
    name: Option<Node<'t>>,
    receiver: Option<(ReceiverRole, Node<'t>)>,
    local: Option<Node<'t>>,
    field: Option<Node<'t>>,
    declared_type: Option<Node<'t>>,
    scope: Option<Node<'t>>,
}

impl<'t> Roles<'t> {
    fn of(vocabulary: &Vocabulary, found: &QueryMatch<'_, 't>) -> Self {
        let mut roles = Self::default();
        for capture in found.captures() {
            let node = capture.node;
            match vocabulary.role(capture.index) {
                Some(Capture::Reference(kind)) => {
                    roles.kind = Some(kind);
                    roles.occurrence = Some(node);
                }
                Some(Capture::Name) => roles.name = Some(node),
                Some(Capture::Receiver(role)) => {
                    roles.receiver = roles.receiver.or(Some((role, node)));
                }
                Some(Capture::Local) => roles.local = Some(node),
                Some(Capture::Field) => roles.field = Some(node),
                Some(Capture::DeclaredType) => roles.declared_type = Some(node),
                Some(Capture::Scope) => roles.scope = Some(node),
                _ => {}
            }
        }
        roles
    }
}

impl<'t> Facts<'t> {
    fn collect(ctx: &SymbolContext<'_>, vocabulary: &Vocabulary, root: Node<'t>) -> Self {
        let mut facts = Self::default();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&vocabulary.query, root, ctx.source.as_bytes());
        while let Some(found) = matches.next() {
            facts.take(ctx, &Roles::of(vocabulary, found));
        }
        facts
    }

    fn take(&mut self, ctx: &SymbolContext<'_>, roles: &Roles<'t>) {
        if let Some(scope) = roles.scope {
            self.scopes.push(scope.byte_range());
        }
        if let (Some(kind), Some(node)) = (roles.kind, roles.name.or(roles.occurrence)) {
            self.occurrences.push(Occurrence { kind, name: node });
        }
        if let (Some((role, receiver)), Some(name)) = (roles.receiver, roles.name) {
            self.see_receiver(ctx, name.byte_range(), role, receiver);
        }
        let declared = roles
            .local
            .map(|node| (node, true))
            .or(roles.field.map(|node| (node, false)));
        if let Some((node, local)) = declared {
            let declared_type = roles
                .declared_type
                .map(|node| text_of(node, ctx.source).trim().to_owned());
            self.declarations.push(RawDeclaration {
                name: text_of(node, ctx.source).trim().to_owned(),
                declared_type,
                at: node.start_byte(),
                local,
            });
        }
    }

    /// Keeps the most specific thing the syntax says about the receiver of a name.
    fn see_receiver(
        &mut self,
        ctx: &SymbolContext<'_>,
        name: Range<usize>,
        role: ReceiverRole,
        node: Node<'_>,
    ) {
        let text = text_of(node, ctx.source).trim().to_owned();
        let seen = match role {
            ReceiverRole::Name => Seen::Name(text, node.start_byte()),
            ReceiverRole::Type => Seen::Type(text),
            ReceiverRole::Member => Seen::Member(text, node.start_byte()),
            ReceiverRole::Own => Seen::Own,
            ReceiverRole::Base => Seen::Base,
            ReceiverRole::Other => Seen::Other,
        };
        let entry = self.receivers.entry(name).or_insert(Seen::Other);
        if matches!(entry, Seen::Other) {
            *entry = seen;
        }
    }
}

/// Finds every occurrence the pack's `references.scm` marks. Occurrences outside any declaration
/// belong to the module symbol of the file. Packs without the query yield no occurrences.
///
/// # Errors
/// A model error if an occurrence has an invalid span, which Tree-sitter never produces.
pub fn extract(
    ctx: &SymbolContext<'_>,
    root: Node<'_>,
    table: &SymbolTable,
) -> Result<Vec<PendingReference>, ParsingError> {
    let Some(vocabulary) = &ctx.pack.references else {
        return Ok(Vec::new());
    };
    let facts = Facts::collect(ctx, vocabulary, root);
    let scopes = Scopes::new(&facts.scopes, facts.declarations.clone());
    let called = called_names(&facts.occurrences);
    let mut seen = HashSet::new();
    let mut pending = Vec::new();
    for occurrence in &facts.occurrences {
        let range = occurrence.name.byte_range();
        // A name that is called is no read: `a?.B()` is a call of `B`, not a read of it as well.
        let is_call_read = occurrence.kind == ReferenceKind::Read && called.contains(&range);
        if is_call_read || !seen.insert((occurrence.kind, range)) {
            continue;
        }
        pending.extend(pending_reference(ctx, table, &facts, &scopes, occurrence)?);
    }
    Ok(pending)
}

/// The byte ranges of the names that some pattern of the query reports as a call.
fn called_names(occurrences: &[Occurrence<'_>]) -> HashSet<Range<usize>> {
    occurrences
        .iter()
        .filter(|o| o.kind == ReferenceKind::Call)
        .map(|o| o.name.byte_range())
        .collect()
}

fn pending_reference(
    ctx: &SymbolContext<'_>,
    table: &SymbolTable,
    facts: &Facts<'_>,
    scopes: &Scopes,
    occurrence: &Occurrence<'_>,
) -> Result<Option<PendingReference>, ParsingError> {
    let node = occurrence.name;
    let name = text_of(node, ctx.source).trim();
    let seen = facts.receivers.get(&node.byte_range());
    let hidden = seen.is_none()
        && scopes
            .find(name, node.start_byte())
            .is_some_and(|d| d.local);
    let (Some(from), Ok(target), false) = (
        table.innermost(node.start_byte()),
        QualifiedName::new(name),
        hidden,
    ) else {
        return Ok(None);
    };
    Ok(Some(PendingReference {
        from: from.id,
        file: ctx.file,
        span: span_of(node)?,
        kind: occurrence.kind,
        target,
        receiver: seen.and_then(|seen| receiver_of(seen, scopes)),
        previous: None,
    }))
}

/// A receiver name that is declared counts through its declared type, an undeclared one by its name.
fn declared_receiver(name: &str, declared: Option<Declared<'_>>) -> Option<Receiver> {
    match declared {
        Some(declared) => Some(
            declared
                .declared_type
                .and_then(|text| QualifiedName::new(text).ok())
                .map_or(Receiver::Unknown, Receiver::Typed),
        ),
        None => QualifiedName::new(name).ok().map(Receiver::Named),
    }
}

/// The receiver as the resolver gets it: a variable counts through its declared type.
fn receiver_of(seen: &Seen, scopes: &Scopes) -> Option<Receiver> {
    match seen {
        Seen::Own => Some(Receiver::Own),
        Seen::Base => Some(Receiver::Base),
        Seen::Other => Some(Receiver::Unknown),
        Seen::Type(name) => QualifiedName::new(name).ok().map(Receiver::Typed),
        Seen::Member(name, at) => declared_receiver(name, scopes.find_member(name, *at)),
        Seen::Name(name, at) => declared_receiver(name, scopes.find(name, *at)),
    }
}
