//! Texts that must never be shown in the prompt of a confirmation.

use codeskimr_domain::Project;

use crate::consts::INVISIBLE_RANGES;

/// Whether `text` holds a control or invisible character: a line break, an escape sequence, a
/// direction override. Such text could make a prompt show something else than what is confirmed.
#[must_use]
pub fn hides_text(text: &str) -> bool {
    text.chars().any(|c| {
        c.is_control()
            || INVISIBLE_RANGES
                .iter()
                .any(|(low, high)| (*low..=*high).contains(&c))
    })
}

/// Whether the name, the location or the branch of `project` holds such a character. For such a
/// project nobody is asked and no window is opened; revoking still works.
#[must_use]
pub fn is_disguised(project: &Project) -> bool {
    let (place, branch) = project.source.place_and_branch();
    [Some(project.name.to_string()), Some(place), branch]
        .into_iter()
        .flatten()
        .any(|text| hides_text(&text))
}
