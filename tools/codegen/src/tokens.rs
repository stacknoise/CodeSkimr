//! The design tokens (blueprint 15.3): reading `design/tokens.toml`, checking that every theme is
//! complete and readable, and the three generated files.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde::Deserialize;

use crate::consts::TOKENS_NOTICE;

/// Smallest contrast ratio of a text/background pair (WCAG AA for normal text).
pub const MIN_CONTRAST: f64 = 4.5;

/// Below this luminance the background of a theme counts as dark.
const DARK_LUMINANCE: f64 = 0.18;

/// The tokens every theme has to define, in the order they are written.
pub const COLOR_TOKENS: [&str; 32] = [
    "bg",
    "surface",
    "line",
    "line-soft",
    "ink",
    "ink-2",
    "muted",
    "faint",
    "accent",
    "accent-bg",
    "accent-hover",
    "accent-fg",
    "good",
    "good-bg",
    "warn",
    "warn-bg",
    "bad",
    "bad-bg",
    "outgoing",
    "highlight",
    "mark",
    "kind-class",
    "kind-interface",
    "kind-record",
    "kind-struct",
    "kind-enum",
    "layer-1",
    "layer-2",
    "layer-3",
    "layer-4",
    "layer-5",
    "layer-6",
];

/// Text on background pairs that must reach [`MIN_CONTRAST`] (blueprint 15.3): the first token
/// is the text, the others are the backgrounds it is used on.
const PAIRS: [(&str, &[&str]); 11] = [
    ("ink", &["bg", "surface"]),
    ("ink-2", &["bg", "surface", "line-soft"]),
    ("muted", &["bg", "surface"]),
    ("faint", &["bg", "surface"]),
    ("accent", &["surface", "bg", "accent-bg"]),
    ("good", &["good-bg", "surface"]),
    ("warn", &["warn-bg", "surface"]),
    ("bad", &["bad-bg", "surface"]),
    ("accent-fg", &["accent"]),
    ("ink", &["accent-bg"]),
    ("outgoing", &["surface"]),
];

/// Further text tokens that are read on `surface`.
const ON_SURFACE: [&str; 11] = [
    "kind-class",
    "kind-interface",
    "kind-record",
    "kind-struct",
    "kind-enum",
    "layer-1",
    "layer-2",
    "layer-3",
    "layer-4",
    "layer-5",
    "layer-6",
];

/// `[meta]`.
#[derive(Debug, Deserialize)]
pub struct Meta {
    /// Themes in the order they are offered.
    pub themes: Vec<String>,
    /// Theme until the user chooses one.
    pub default_theme: String,
    /// Theme until the user chooses one, on a dark system.
    pub dark_theme: String,
    /// Key of the `localStorage` mirror.
    pub storage_key: String,
}

/// `[shared]`.
#[derive(Debug, Deserialize)]
pub struct Shared {
    /// Font of the text.
    pub font_sans: String,
    /// Font of code and names.
    pub font_mono: String,
    /// Corner radii in pixels, by use.
    pub radius: BTreeMap<String, u32>,
    /// Spacings in pixels.
    pub spacing: Vec<u32>,
}

/// The whole file.
#[derive(Debug, Deserialize)]
pub struct Tokens {
    /// Which themes exist.
    pub meta: Meta,
    /// What does not depend on a theme.
    pub shared: Shared,
    /// One table of colours per theme.
    pub themes: BTreeMap<String, BTreeMap<String, String>>,
}

impl Tokens {
    /// Reads the text of `design/tokens.toml`.
    ///
    /// # Errors
    /// The parser's message if the text is not valid.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    fn color_names() -> impl Iterator<Item = &'static str> {
        COLOR_TOKENS.iter().copied()
    }

    /// Every problem with the structure and the contrast of the themes.
    ///
    /// # Errors
    /// One message per problem.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut problems = self.structure_problems();
        if problems.is_empty() {
            for name in &self.meta.themes {
                problems.extend(self.contrast_problems(name));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }

    fn structure_problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for name in [&self.meta.default_theme, &self.meta.dark_theme] {
            if !self.meta.themes.contains(name) {
                problems.push(format!("meta: theme {name:?} is not in meta.themes"));
            }
        }
        for name in self.themes.keys().filter(|n| !self.meta.themes.contains(n)) {
            problems.push(format!("themes.{name}: not listed in meta.themes"));
        }
        for name in &self.meta.themes {
            let Some(colors) = self.themes.get(name) else {
                problems.push(format!("themes.{name}: missing"));
                continue;
            };
            for token in Self::color_names().filter(|t| !colors.contains_key(*t)) {
                problems.push(format!("themes.{name}: token {token} is missing"));
            }
            for (token, value) in colors {
                if !Self::color_names().any(|t| t == token) {
                    problems.push(format!("themes.{name}: unknown token {token}"));
                } else if rgb(value).is_none() {
                    problems.push(format!("themes.{name}.{token}: {value:?} is not #RRGGBB"));
                }
            }
        }
        problems
    }

    fn contrast_problems(&self, theme: &str) -> Vec<String> {
        let Some(colors) = self.themes.get(theme) else {
            return Vec::new();
        };
        let mut problems = Vec::new();
        let mut check = |text: &str, background: &str| {
            let ratio = colors
                .get(text)
                .zip(colors.get(background))
                .and_then(|(a, b)| Some(contrast(rgb(a)?, rgb(b)?)));
            if let Some(ratio) = ratio.filter(|r| *r < MIN_CONTRAST) {
                problems.push(format!(
                    "themes.{theme}: {text} on {background} has contrast {ratio:.2}, needs {MIN_CONTRAST}"
                ));
            }
        };
        for (text, backgrounds) in PAIRS {
            for background in backgrounds {
                check(text, background);
            }
        }
        for text in ON_SURFACE {
            check(text, "surface");
        }
        problems
    }
}

/// `#RRGGBB` as three channels.
fn rgb(text: &str) -> Option<[u8; 3]> {
    let digits = text.strip_prefix('#').filter(|d| d.len() == 6)?;
    let channel = |i: usize| u8::from_str_radix(digits.get(i * 2..i * 2 + 2)?, 16).ok();
    Some([channel(0)?, channel(1)?, channel(2)?])
}

/// Relative luminance after WCAG 2.1.
fn luminance(color: [u8; 3]) -> f64 {
    let linear = |c: u8| {
        let s = f64::from(c) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2])
}

/// Contrast ratio of two colours, 1 to 21.
#[must_use]
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `tokens.css`: the variables of every theme, scoped by `data-theme`.
#[must_use]
pub fn render_css(tokens: &Tokens) -> String {
    let mut out = format!("/* {TOKENS_NOTICE} */\n\n:root {{\n");
    let _ = writeln!(
        out,
        "  --font-sans: \"{}\", system-ui, sans-serif;",
        tokens.shared.font_sans
    );
    let _ = writeln!(
        out,
        "  --font-mono: \"{}\", ui-monospace, monospace;",
        tokens.shared.font_mono
    );
    for (name, px) in &tokens.shared.radius {
        let _ = writeln!(out, "  --radius-{name}: {px}px;");
    }
    for px in &tokens.shared.spacing {
        let _ = writeln!(out, "  --space-{px}: {px}px;");
    }
    out.push_str("}\n");
    for name in &tokens.meta.themes {
        let _ = write!(
            out,
            "\n[data-theme=\"{name}\"] {{\n  color-scheme: {};\n",
            scheme(tokens, name)
        );
        for token in Tokens::color_names() {
            if let Some(value) = tokens.themes.get(name).and_then(|c| c.get(token)) {
                let _ = writeln!(out, "  --{token}: {value};");
            }
        }
        out.push_str("}\n");
    }
    out
}

/// Whether a theme is drawn on a dark background, for the browser's own controls.
fn scheme(tokens: &Tokens, theme: &str) -> &'static str {
    let dark = tokens
        .themes
        .get(theme)
        .and_then(|c| c.get("bg"))
        .and_then(|bg| rgb(bg))
        .is_some_and(|bg| luminance(bg) < DARK_LUMINANCE);
    if dark { "dark" } else { "light" }
}

/// `tokens.ts`: the theme names and the token names for the code.
#[must_use]
pub fn render_ts(tokens: &Tokens) -> String {
    let names = |list: &[String]| {
        list.iter()
            .map(|n| format!("{n:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!("// {TOKENS_NOTICE}\n\n");
    let _ = writeln!(out, "/** The themes, in the order they are offered. */");
    let _ = writeln!(
        out,
        "export const THEMES = [{}] as const;\n",
        names(&tokens.meta.themes)
    );
    out.push_str("/** Name of a theme. */\nexport type ThemeName = (typeof THEMES)[number];\n\n");
    let _ = writeln!(
        out,
        "/** Theme until the user chooses one. */\nexport const DEFAULT_THEME: ThemeName = {:?};\n",
        tokens.meta.default_theme
    );
    let _ = writeln!(
        out,
        "/** Theme until the user chooses one, on a dark system. */\nexport const DARK_THEME: ThemeName = {:?};\n",
        tokens.meta.dark_theme
    );
    let _ = writeln!(
        out,
        "/** Key of the mirror of the choice in `localStorage`. */\nexport const THEME_STORAGE_KEY = {:?};\n",
        tokens.meta.storage_key
    );
    let colors: Vec<String> = Tokens::color_names().map(str::to_owned).collect();
    let _ = writeln!(
        out,
        "/** The colour tokens every theme defines. */\nexport const COLOR_TOKENS = [{}] as const;",
        names(&colors)
    );
    out
}

/// `theme-init.js`: sets `data-theme` before the first paint. It is a file of its own, so that
/// the page needs no inline script (ADR 0028).
#[must_use]
pub fn render_init(tokens: &Tokens) -> String {
    let themes = tokens
        .meta
        .themes
        .iter()
        .map(|n| format!("{n:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "// {TOKENS_NOTICE}\n(function () {{\n  var themes = [{themes}];\n  var chosen = null;\n  try {{\n    chosen = localStorage.getItem({:?});\n  }} catch (error) {{\n    chosen = null;\n  }}\n  var dark = window.matchMedia && window.matchMedia(\"(prefers-color-scheme: dark)\").matches;\n  var fallback = dark ? {:?} : {:?};\n  var theme = themes.indexOf(chosen) >= 0 ? chosen : fallback;\n  document.documentElement.setAttribute(\"data-theme\", theme);\n}})();\n",
        tokens.meta.storage_key, tokens.meta.dark_theme, tokens.meta.default_theme
    )
}
