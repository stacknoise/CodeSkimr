//! The descriptor syntax of SCIP symbols (`scip.proto`, message `Symbol`): the part of a symbol
//! after its four header fields is a chain of descriptors, each a name and a suffix. The import
//! needs the last descriptor, which names the symbol itself.

use crate::consts::{
    CLOSE_BRACKET, CLOSE_PAREN, LOCAL_PREFIX, NAME_DOLLAR, NAME_EXTRA_CHARS, NAME_QUOTE,
    OPEN_BRACKET, OPEN_PAREN, SUFFIX_MACRO, SUFFIX_META, SUFFIX_NAMESPACE, SUFFIX_TERM,
    SUFFIX_TYPE, SYMBOL_HEADER_FIELDS, SYMBOL_SEPARATOR,
};

/// What a descriptor describes, from its suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorKind {
    /// `name/`: namespace or package.
    Namespace,
    /// `name#`: class, struct, interface, trait, enum and the like.
    Type,
    /// `name.`: field, property, constant, enum member.
    Term,
    /// `name(disambiguator).`: function or method; overloads differ in the disambiguator.
    Method,
    /// `[name]`: type parameter.
    TypeParameter,
    /// `(name)`: parameter.
    Parameter,
    /// `name:`: meta.
    Meta,
    /// `name!`: macro.
    Macro,
}

/// The last descriptor of a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Descriptor {
    /// The name without quotes.
    pub(crate) name: String,
    /// What it describes.
    pub(crate) kind: DescriptorKind,
}

/// Whether the symbol is local to one document (`local 12`), which has no definition elsewhere.
pub(crate) fn is_local(symbol: &str) -> bool {
    symbol.starts_with(LOCAL_PREFIX)
}

/// The last descriptor of a global symbol, or `None` for a local or malformed symbol.
pub(crate) fn last_descriptor(symbol: &str) -> Option<Descriptor> {
    last_two(symbol).map(|(_, last)| last)
}

/// The last descriptor of a global symbol and the one before it, if there is one.
pub(crate) fn last_two(symbol: &str) -> Option<(Option<Descriptor>, Descriptor)> {
    if is_local(symbol) {
        return None;
    }
    let mut chars = symbol.chars().peekable();
    for _ in 0..SYMBOL_HEADER_FIELDS {
        skip_header_field(&mut chars);
    }
    let (mut before, mut last) = (None, None);
    while chars.peek().is_some() {
        before = last.replace(descriptor(&mut chars)?);
    }
    last.map(|last| (before, last))
}

pub(crate) type Chars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

/// Skips one header field and its separator; a doubled separator belongs to the field.
fn skip_header_field(chars: &mut Chars<'_>) {
    while let Some(c) = chars.next() {
        if c == SYMBOL_SEPARATOR {
            if chars.peek() == Some(&SYMBOL_SEPARATOR) {
                chars.next();
            } else {
                return;
            }
        }
    }
}

pub(crate) fn is_simple(c: char) -> bool {
    c.is_alphanumeric() || c == NAME_DOLLAR || NAME_EXTRA_CHARS.contains(&c)
}

/// A name: quoted (a doubled quote stands for one) or a run of simple characters.
pub(crate) fn name(chars: &mut Chars<'_>) -> Option<String> {
    let mut text = String::new();
    if chars.peek() == Some(&NAME_QUOTE) {
        chars.next();
        loop {
            let c = chars.next()?;
            if c != NAME_QUOTE {
                text.push(c);
            } else if chars.peek() == Some(&NAME_QUOTE) {
                chars.next();
                text.push(NAME_QUOTE);
            } else {
                break;
            }
        }
    } else {
        while let Some(c) = chars.next_if(|c| is_simple(*c)) {
            text.push(c);
        }
    }
    (!text.is_empty()).then_some(text)
}

/// Reads up to and including `close`.
fn skip_through(chars: &mut Chars<'_>, close: char) -> Option<()> {
    chars.by_ref().find(|c| *c == close).map(drop)
}

fn bracketed(chars: &mut Chars<'_>, close: char, kind: DescriptorKind) -> Option<Descriptor> {
    let name = name(chars)?;
    (chars.next()? == close).then_some(Descriptor { name, kind })
}

fn descriptor(chars: &mut Chars<'_>) -> Option<Descriptor> {
    match *chars.peek()? {
        OPEN_PAREN => {
            chars.next();
            bracketed(chars, CLOSE_PAREN, DescriptorKind::Parameter)
        }
        OPEN_BRACKET => {
            chars.next();
            bracketed(chars, CLOSE_BRACKET, DescriptorKind::TypeParameter)
        }
        _ => named(chars),
    }
}

pub(crate) fn named(chars: &mut Chars<'_>) -> Option<Descriptor> {
    let name = name(chars)?;
    let kind = match chars.next()? {
        SUFFIX_NAMESPACE => DescriptorKind::Namespace,
        SUFFIX_TYPE => DescriptorKind::Type,
        SUFFIX_TERM => DescriptorKind::Term,
        SUFFIX_META => DescriptorKind::Meta,
        SUFFIX_MACRO => DescriptorKind::Macro,
        OPEN_PAREN => {
            skip_through(chars, CLOSE_PAREN)?;
            (chars.next()? == SUFFIX_TERM).then_some(DescriptorKind::Method)?
        }
        _ => return None,
    };
    Some(Descriptor { name, kind })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn last(symbol: &str) -> Option<(String, DescriptorKind)> {
        last_descriptor(symbol).map(|d| (d.name, d.kind))
    }

    const CASES: [(&str, &str, DescriptorKind); 10] = [
        (
            "scip-dotnet nuget . . ns/Svc#Run().",
            "Run",
            DescriptorKind::Method,
        ),
        (
            "scip-dotnet nuget . . ns/Svc#Zero(+6).",
            "Zero",
            DescriptorKind::Method,
        ),
        (
            "scip-dotnet nuget . . ns/Svc#Name.",
            "Name",
            DescriptorKind::Term,
        ),
        ("scip-dotnet nuget . . ns/A#B#", "B", DescriptorKind::Type),
        ("scip-dotnet nuget . . ns/", "ns", DescriptorKind::Namespace),
        (
            "scip-dotnet nuget . . ns/Svc#Run().(count)",
            "count",
            DescriptorKind::Parameter,
        ),
        (
            "scip-dotnet nuget . . ns/Svc#[T]",
            "T",
            DescriptorKind::TypeParameter,
        ),
        (
            "scip-dotnet nuget . . ns/Svc#`.ctor`().",
            ".ctor",
            DescriptorKind::Method,
        ),
        (
            "scip-dotnet nuget . . ns/Svc#`a``b`.",
            "a`b",
            DescriptorKind::Term,
        ),
        (
            "rust-analyzer cargo my  crate 0.1.0 impl#[Store]save().",
            "save",
            DescriptorKind::Method,
        ),
    ];

    #[test]
    fn the_last_descriptor_names_the_symbol() {
        for (symbol, name, kind) in CASES {
            assert_eq!(last(symbol), Some((name.to_owned(), kind)), "{symbol}");
        }
    }

    #[test]
    fn local_and_malformed_symbols_have_no_descriptor() {
        assert!(is_local("local 12"));
        for symbol in [
            "local 12",
            "",
            "a b c d",
            "a b c d ns/Svc",
            "a b c d `open",
            "a b c d x(",
        ] {
            assert_eq!(last(symbol), None, "{symbol:?}");
        }
    }
}
