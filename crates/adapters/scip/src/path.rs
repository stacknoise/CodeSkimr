//! Paths of an index are untrusted: they are cleaned lexically and refused if they could leave
//! the project root (blueprint S-05). The import never opens a file by such a path; it only
//! compares it with the paths of the snapshot.

use codeskimr_domain::RelativePath;

use crate::consts::{
    DRIVE_COMPONENT_CHARS, DRIVE_MARK, MAX_PATH_BYTES, PATH_CURRENT, PATH_FORBIDDEN, PATH_JOIN,
    PATH_PARENT, PATH_SEPARATORS,
};
use crate::error::Corruption;

fn is_drive(component: &str) -> bool {
    let mut chars = component.chars();
    component.chars().count() == DRIVE_COMPONENT_CHARS
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.next() == Some(DRIVE_MARK)
}

/// The relative path in its clean form: `/` as the only separator, no empty and no `.`
/// components.
///
/// # Errors
/// [`Corruption::UnsafePath`] if the path is empty, starts with a separator, names a drive,
/// holds a `..` component or a NUL character; [`Corruption::TextTooLong`] above the path limit.
pub fn normalize_path(raw: &str) -> Result<RelativePath, Corruption> {
    if raw.len() > MAX_PATH_BYTES {
        return Err(Corruption::TextTooLong);
    }
    if raw.starts_with(PATH_SEPARATORS) || raw.contains(PATH_FORBIDDEN) {
        return Err(Corruption::UnsafePath);
    }
    let mut kept = Vec::new();
    for component in raw.split(PATH_SEPARATORS) {
        if component == PATH_PARENT || is_drive(component) {
            return Err(Corruption::UnsafePath);
        }
        if !component.is_empty() && component != PATH_CURRENT {
            kept.push(component);
        }
    }
    RelativePath::new(kept.join(PATH_JOIN)).map_err(|_| Corruption::UnsafePath)
}
