//! Fully qualified names, shared by symbols and layers.

use crate::consts::{MODULE_NAME_SEPARATOR, NAME_SEPARATOR};
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

    /// Whether this name is `outer` or lies inside it: `outer` is the first segments of the name,
    /// cut at a separator (`Shop` holds `Shop.Ui`, but not `Shopping`).
    #[must_use]
    pub fn is_inside(&self, outer: &str) -> bool {
        self.as_str().strip_prefix(outer).is_some_and(|rest| {
            rest.is_empty()
                || rest.starts_with(NAME_SEPARATOR)
                || rest.starts_with(MODULE_NAME_SEPARATOR)
        })
    }
}
