//! Fully qualified names, shared by symbols and layers.

use crate::consts::NAME_SEPARATOR;
use crate::vocabulary::text_type;

text_type!(
    /// Fully qualified name of a symbol, such as `Namespace.Type.member`.
    QualifiedName
);

impl QualifiedName {
    /// The name without its last segment (`App.Domain` for `App.Domain.Order`), `None` for a
    /// name that has only one segment.
    #[must_use]
    pub fn enclosing(&self) -> Option<&str> {
        self.as_str()
            .rsplit_once(NAME_SEPARATOR)
            .map(|(enclosing, _)| enclosing)
    }
}
