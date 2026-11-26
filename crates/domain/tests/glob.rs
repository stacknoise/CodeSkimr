//! The glob matcher of the exception lists (ADR 0101).

use codeskimr_domain::glob_matches;

#[test]
fn a_star_stands_for_any_run_also_across_separators() {
    assert!(glob_matches(
        "Data/Migrations/**",
        "Data/Migrations/2026/Init.cs"
    ));
    assert!(glob_matches(
        "*.DesignTimeFactory",
        "Data.DesignTimeFactory"
    ));
    assert!(glob_matches("Migrations.*.Up", "Migrations.V1.Up"));
    assert!(glob_matches("*", ""));
    assert!(!glob_matches("Data/*", "Api/Data/x"));
}

#[test]
fn a_question_mark_stands_for_one_character_and_the_rest_for_itself() {
    assert!(glob_matches("V?", "V1"));
    assert!(!glob_matches("V?", "V"));
    assert!(!glob_matches("V?", "V12"));
    assert!(glob_matches("exact", "exact"));
    assert!(!glob_matches("exact", "exacts"));
    assert!(glob_matches("a*b*c", "aXXbYYc"));
    assert!(!glob_matches("a*b*c", "aXXbYY"));
}

#[test]
fn backtracking_and_non_ascii_text_match_as_a_whole() {
    assert!(glob_matches("*ab", "aab"));
    assert!(glob_matches("a*a*a", "aaaa"));
    assert!(!glob_matches("a*a*a", "aa"));
    assert!(glob_matches("*b?d", "abcbxd"));
    assert!(glob_matches("Gr?ße*", "Größe.Wert"));
    assert!(!glob_matches("", "x"));
    assert!(glob_matches("", ""));
    assert!(glob_matches("a**", "a"));
    assert!(!glob_matches("?*", ""));
}
