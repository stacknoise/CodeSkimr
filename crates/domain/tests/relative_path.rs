//! Placing a relative path in a folder (workspaces of level 2, ADR 0099).

use codeskimr_domain::{ModelError, RelativePath};

#[test]
fn a_path_is_placed_in_a_folder() -> Result<(), ModelError> {
    let placed = RelativePath::new("src/a.rs")?.in_folder(&RelativePath::new("cli")?);
    assert_eq!(placed, Some(RelativePath::new("cli/src/a.rs")?));
    Ok(())
}

#[test]
fn a_path_is_below_a_folder_only_by_whole_names() -> Result<(), ModelError> {
    let cli = RelativePath::new("cli")?;
    assert!(RelativePath::new("cli/a.rs")?.is_below(&cli));
    assert!(RelativePath::new("cli/src/a.rs")?.is_below(&cli));
    assert!(!RelativePath::new("client/a.rs")?.is_below(&cli));
    assert!(!cli.is_below(&cli));
    Ok(())
}
