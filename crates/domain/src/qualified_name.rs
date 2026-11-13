//! Fully qualified names, shared by symbols and layers.

use crate::vocabulary::text_type;

text_type!(
    /// Fully qualified name of a symbol, such as `Namespace.Type.member`.
    QualifiedName
);
