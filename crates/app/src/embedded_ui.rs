//! The built UI inside the program: the files of `web/dist` as they were when the program was
//! built (see `build.rs`). The list is empty if the UI was not built first.

#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/embedded_ui.rs"));

/// Unit tests never see a built UI, so their results do not depend on whether `web/` was built.
#[cfg(test)]
pub static FILES: &[(&str, &[u8])] = &[];
