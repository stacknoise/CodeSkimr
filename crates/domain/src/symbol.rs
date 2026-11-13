//! Named things in the code: classes, functions, fields and so on.

use crate::consts::{
    KIND_CLASS, KIND_CONSTANT, KIND_CONSTRUCTOR, KIND_ENUM, KIND_ENUM_MEMBER, KIND_EVENT,
    KIND_FIELD, KIND_FUNCTION, KIND_INTERFACE, KIND_METHOD, KIND_MODULE, KIND_NAMESPACE,
    KIND_PARAMETER, KIND_PROPERTY, KIND_RECORD, KIND_STRUCT, KIND_TRAIT, KIND_TYPE_ALIAS,
    KIND_UNION, VISIBILITY_INTERNAL, VISIBILITY_PRIVATE, VISIBILITY_PROTECTED, VISIBILITY_PUBLIC,
};
use crate::file::FileId;
use crate::qualified_name::QualifiedName;
use crate::span::Span;
use crate::vocabulary::{id_type, labeled_enum, text_type};

id_type!(
    /// Identifies a symbol within a snapshot.
    SymbolId
);
text_type!(
    /// Simple name of a symbol.
    SymbolName
);

labeled_enum!(
    /// Language-neutral kind of a symbol. Language packs map their syntax onto these.
    SymbolKind {
        /// File-level container; free functions belong to it.
        Module => KIND_MODULE,
        /// Namespace or package.
        Namespace => KIND_NAMESPACE,
        /// Class.
        Class => KIND_CLASS,
        /// Interface.
        Interface => KIND_INTERFACE,
        /// Trait.
        Trait => KIND_TRAIT,
        /// Record.
        Record => KIND_RECORD,
        /// Struct.
        Struct => KIND_STRUCT,
        /// Enumeration.
        Enum => KIND_ENUM,
        /// Union.
        Union => KIND_UNION,
        /// Free function.
        Function => KIND_FUNCTION,
        /// Method of a type.
        Method => KIND_METHOD,
        /// Constructor.
        Constructor => KIND_CONSTRUCTOR,
        /// Property.
        Property => KIND_PROPERTY,
        /// Field.
        Field => KIND_FIELD,
        /// Constant.
        Constant => KIND_CONSTANT,
        /// Event.
        Event => KIND_EVENT,
        /// Member of an enumeration.
        EnumMember => KIND_ENUM_MEMBER,
        /// Parameter.
        Parameter => KIND_PARAMETER,
        /// Type alias.
        TypeAlias => KIND_TYPE_ALIAS,
    }
);

labeled_enum!(
    /// Who can see a symbol.
    Visibility {
        /// Visible everywhere.
        Public => VISIBILITY_PUBLIC,
        /// Visible in the type and its subtypes.
        Protected => VISIBILITY_PROTECTED,
        /// Visible in the same package or assembly.
        Internal => VISIBILITY_INTERNAL,
        /// Visible only in the declaring scope.
        Private => VISIBILITY_PRIVATE,
    }
);

impl SymbolKind {
    /// Whether the symbol has a body that is measured: function, method or constructor.
    #[must_use]
    pub const fn is_callable(self) -> bool {
        matches!(self, Self::Function | Self::Method | Self::Constructor)
    }

    /// Whether the symbol is a type that holds members.
    #[must_use]
    pub const fn is_type(self) -> bool {
        matches!(
            self,
            Self::Class
                | Self::Interface
                | Self::Trait
                | Self::Record
                | Self::Struct
                | Self::Enum
                | Self::Union
        )
    }

    /// Whether callables roll up into the symbol: a type, module or namespace (ADR 0015).
    #[must_use]
    pub const fn is_rollup_target(self) -> bool {
        self.is_type() || matches!(self, Self::Module | Self::Namespace)
    }
}

/// Anything with a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// Identity.
    pub id: SymbolId,
    /// File that declares it.
    pub file: FileId,
    /// Kind.
    pub kind: SymbolKind,
    /// Simple name.
    pub name: SymbolName,
    /// Fully qualified name.
    pub qualified_name: QualifiedName,
    /// Enclosing symbol, absent for top-level modules.
    pub parent: Option<SymbolId>,
    /// Visibility; `None` where the language has no such concept or it is unknown.
    pub visibility: Option<Visibility>,
    /// Whether the symbol is static.
    pub is_static: bool,
    /// Declaration range.
    pub span: Span,
    /// Signature as written, if any.
    pub signature: Option<String>,
    /// Documentation comment, if any.
    pub doc: Option<String>,
}
