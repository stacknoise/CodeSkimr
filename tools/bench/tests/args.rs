//! Tests of the command-line parsing.

use std::error::Error;

use codeskimr_bench::BenchError;
use codeskimr_bench::args::parse;

fn args(items: &[&str]) -> impl Iterator<Item = String> {
    items
        .iter()
        .map(|item| (*item).to_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

#[test]
fn defaults_and_flags() -> Result<(), Box<dyn Error>> {
    assert!(!parse(args(&[]))?.no_base);
    let parsed = parse(args(&["--base", "a", "--new", "b", "--no-base"]))?;
    assert_eq!(
        (parsed.base.as_str(), parsed.new.as_str(), parsed.no_base),
        ("a", "b", true)
    );
    Ok(())
}

#[test]
fn unknown_flag_is_named_as_unknown() {
    assert!(matches!(
        parse(args(&["--wat", "x"])),
        Err(BenchError::UnknownFlag(_))
    ));
}

#[test]
fn flag_without_value_is_reported() {
    assert!(matches!(
        parse(args(&["--base"])),
        Err(BenchError::MissingFlagValue(_))
    ));
}
