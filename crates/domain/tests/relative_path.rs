//! Placing a relative path in a folder (workspaces of level 2, ADR 0099).

use codeskimr_domain::RelativePath;

fn path(text: &str) -> RelativePath {
    RelativePath::new(text).expect("path")
}

#[test]
fn a_path_is_placed_in_a_folder() {
    let placed = path("src/a.rs").in_folder(&path("cli"));
    assert_eq!(placed, Some(path("cli/src/a.rs")));
}

#[test]
fn a_path_is_below_a_folder_only_by_whole_names() {
    assert!(path("cli/a.rs").is_below(&path("cli")));
    assert!(path("cli/src/a.rs").is_below(&path("cli")));
    assert!(!path("client/a.rs").is_below(&path("cli")));
    assert!(!path("cli").is_below(&path("cli")));
}
