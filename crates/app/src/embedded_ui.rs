//! The built UI inside the program: the files of `web/dist` as they were when the program was
//! built (see `build.rs`). The list is empty if the UI was not built first.

include!(concat!(env!("OUT_DIR"), "/embedded_ui.rs"));
