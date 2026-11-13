//! Tests of the design token checks and the generated files.

use codeskimr_codegen::tokens::{Tokens, contrast, render_css, render_init, render_ts};

const REAL: &str = include_str!("../../../design/tokens.toml");

#[cfg(test)]
fn real() -> Tokens {
    Tokens::parse(REAL).expect("the checked-in tokens parse")
}

#[cfg(test)]
fn problems(text: &str) -> Vec<String> {
    Tokens::parse(text)
        .expect("parses")
        .validate()
        .err()
        .unwrap_or_default()
}

#[test]
fn the_checked_in_tokens_are_complete_and_readable() {
    assert_eq!(real().validate(), Ok(()));
}

#[test]
fn a_missing_token_is_found_in_the_theme_that_lacks_it() {
    let text = REAL.replacen("mark = \"#6B5A12\"\n", "", 1);
    let found = problems(&text);
    assert_eq!(found, ["themes.dunkel: token mark is missing"]);
}

#[test]
fn an_unknown_token_and_a_bad_colour_are_found() {
    let text = REAL.replacen(
        "mark = \"#FDE68A\"",
        "mark = \"yellow\"\nextra = \"#000000\"",
        1,
    );
    let found = problems(&text);
    assert!(
        found.iter().any(|p| p.contains("unknown token extra")),
        "{found:?}"
    );
    assert!(found.iter().any(|p| p.contains("not #RRGGBB")), "{found:?}");
}

#[test]
fn a_pair_below_4_5_to_1_fails_with_the_pair_named() {
    // The value #E7F6EC of the first mock reached only 4.49:1 with `good` (blueprint 15.3).
    let text = REAL.replacen("good-bg = \"#EAF7EF\"", "good-bg = \"#E7F6EC\"", 1);
    let found = problems(&text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("themes.hell: good on good-bg"),
        "{found:?}"
    );
}

#[test]
fn themes_must_be_listed_and_the_defaults_must_exist() {
    let unlisted = REAL.replacen(
        "themes = [\"hell\", \"petrol\", \"dunkel\"]",
        "themes = [\"hell\", \"petrol\"]",
        1,
    );
    assert!(problems(&unlisted).iter().any(|p| p.contains("not listed")));
    let missing = REAL.replacen("default_theme = \"petrol\"", "default_theme = \"rot\"", 1);
    assert!(problems(&missing).iter().any(|p| p.contains("\"rot\"")));
}

#[test]
fn contrast_follows_wcag() {
    assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 1e-9);
    assert!((contrast([255, 255, 255], [255, 255, 255]) - 1.0).abs() < 1e-9);
}

#[test]
fn the_css_has_every_theme_scoped_and_every_token() {
    let tokens = real();
    let css = render_css(&tokens);
    for theme in ["hell", "petrol", "dunkel"] {
        assert!(
            css.contains(&format!("[data-theme=\"{theme}\"] {{")),
            "{theme}"
        );
    }
    assert_eq!(css.matches("--accent-fg:").count(), 3);
    assert!(css.contains("color-scheme: dark"));
    assert!(css.contains("--radius-dialog: 14px;"));
    assert!(css.contains("--space-32: 32px;"));
}

#[test]
fn the_script_reads_the_mirror_and_the_system_and_needs_no_inline_code() {
    let script = render_init(&real());
    assert!(script.contains("localStorage.getItem(\"codeskimr.theme\")"));
    assert!(script.contains("prefers-color-scheme: dark"));
    assert!(script.contains("\"dunkel\" : \"petrol\""));
    let ts = render_ts(&real());
    assert!(ts.contains("export const THEMES = [\"hell\", \"petrol\", \"dunkel\"] as const;"));
}
