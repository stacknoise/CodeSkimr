//! The symbols rust-analyzer gives to impl blocks. They are no descriptors of a type the project
//! has a definition for, so they are read apart: `impl#[Type]` is what `Self` names inside the
//! block, and a member of `impl#[Type][Trait]` is the implementation of a method of the trait
//! (ADR 0033, decision 2).

use crate::consts::{
    CLOSE_BRACKET, IMPL_MARKER, OPEN_BRACKET, OPEN_GENERIC, PATH_SEPARATOR, RELATIVE_SEGMENTS,
    ROOT_SEGMENT, SUFFIX_NAMESPACE, SUFFIX_TYPE, SYMBOL_SEPARATOR,
};
use crate::symbol::{Chars, DescriptorKind, header_len, is_simple, name, named};

/// A type or trait as rust-analyzer writes it inside the brackets, without generic arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeName {
    /// The modules in front of the name, as written (`std::io` for `std::io::Error`); a leading
    /// `crate` is not part of it.
    pub(crate) path: Vec<String>,
    /// The path starts at the crate root (`crate::a::Type`): the type lies in exactly these modules.
    pub(crate) rooted: bool,
    /// The name of the type or trait.
    pub(crate) name: String,
}

/// What an impl symbol stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImplSymbol {
    /// The impl block itself, as `Self` names it. `prefix` is the symbol text before `impl#`, so
    /// the symbol of the type in the same module is `prefix` + name + `#`.
    SelfType {
        /// The symbol text up to `impl#`.
        prefix: String,
        /// The type the block is for.
        ty: TypeName,
    },
    /// A method in a trait implementation.
    TraitMember {
        /// The trait.
        trait_ty: TypeName,
        /// The name of the method.
        member: String,
    },
}

/// Reads a type or trait written as text: generic arguments are cut off (`Job<Queued>`), a path
/// stays. `None` if it is not a plain path of names (`&Type`, `[T; N]`, `dyn Any`) or starts with
/// `self` or `super`, which would need the module of the block.
fn type_name(text: &str) -> Option<TypeName> {
    let base = text.split(OPEN_GENERIC).next()?;
    let mut segments: Vec<&str> = base.split(PATH_SEPARATOR).collect();
    let rooted = segments.first() == Some(&ROOT_SEGMENT);
    if rooted {
        segments.remove(0);
    }
    let last = segments.pop()?;
    let plain = |segment: &&str| !segment.is_empty() && segment.chars().all(is_simple);
    let relative =
        segments.iter().any(|s| RELATIVE_SEGMENTS.contains(s)) || RELATIVE_SEGMENTS.contains(&last);
    (segments.iter().all(&plain) && plain(&last) && !relative).then(|| TypeName {
        path: segments.into_iter().map(str::to_owned).collect(),
        rooted,
        name: last.to_owned(),
    })
}

/// `[name]`; the opening bracket is already read. The name is quoted if it is not simple.
fn bracket(chars: &mut Chars<'_>) -> Option<TypeName> {
    let text = name(chars)?;
    (chars.next()? == CLOSE_BRACKET)
        .then(|| type_name(&text))
        .flatten()
}

/// Reads the impl symbol `text`, or `None` if it is not one or has a form the import does not
/// know. The header (package name) is skipped, so a package called `impl#[X]` is no impl block.
pub(crate) fn parse(text: &str) -> Option<ImplSymbol> {
    let start = header_len(text);
    let at = start + text.get(start..)?.find(IMPL_MARKER)?;
    let prefix = text.get(..at)?;
    if !prefix.ends_with([SYMBOL_SEPARATOR, SUFFIX_NAMESPACE, SUFFIX_TYPE]) {
        return None;
    }
    let mut chars = text.get(at + IMPL_MARKER.len()..)?.chars().peekable();
    let mut names = vec![bracket(&mut chars)?];
    while chars.next_if_eq(&OPEN_BRACKET).is_some() {
        names.push(bracket(&mut chars)?);
    }
    let mut names = names.into_iter();
    let (first, second) = (names.next()?, names.next());
    if chars.peek().is_none() {
        return Some(ImplSymbol::SelfType {
            prefix: prefix.to_owned(),
            ty: first,
        });
    }
    let member = named(&mut chars)?;
    (chars.peek().is_none() && member.kind == DescriptorKind::Method).then_some(
        ImplSymbol::TraitMember {
            trait_ty: second?,
            member: member.name,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "rust-analyzer cargo c 0.1.0 ";

    fn ty(path: &[&str], rooted: bool, name: &str) -> TypeName {
        TypeName {
            path: path.iter().map(|s| (*s).to_owned()).collect(),
            rooted,
            name: name.to_owned(),
        }
    }

    fn self_type(prefix: &str, ty: TypeName) -> ImplSymbol {
        ImplSymbol::SelfType {
            prefix: prefix.to_owned(),
            ty,
        }
    }

    fn member(trait_ty: TypeName, member: &str) -> ImplSymbol {
        ImplSymbol::TraitMember {
            trait_ty,
            member: member.to_owned(),
        }
    }

    #[test]
    fn the_block_of_a_type_and_of_a_trait_implementation_name_the_type() {
        assert_eq!(
            parse(&format!("{HEAD}fingerprint/impl#[Fingerprints]")),
            Some(self_type(
                &format!("{HEAD}fingerprint/"),
                ty(&[], false, "Fingerprints")
            ))
        );
        assert_eq!(
            parse(&format!("{HEAD}impl#[Store][CodeIndex]")),
            Some(self_type(HEAD, ty(&[], false, "Store")))
        );
    }

    #[test]
    fn generic_arguments_are_cut_off_and_paths_are_kept() {
        assert_eq!(
            parse(&format!("{HEAD}impl#[`Job<Queued>`]")),
            Some(self_type(HEAD, ty(&[], false, "Job")))
        );
        assert_eq!(
            parse(&format!("{HEAD}impl#[`crate::a::Repository`]")),
            Some(self_type(HEAD, ty(&["a"], true, "Repository")))
        );
        assert_eq!(
            parse(&format!("{HEAD}impl#[`io::Error`]")),
            Some(self_type(HEAD, ty(&["io"], false, "Error")))
        );
        assert_eq!(
            parse(&format!("{HEAD}impl#[CodedError][`From<StorageError>`]")),
            Some(self_type(HEAD, ty(&[], false, "CodedError")))
        );
        assert_eq!(
            parse(&format!("{HEAD}impl#[Store][`std::hash::Hash`]hash().")),
            Some(member(ty(&["std", "hash"], false, "Hash"), "hash"))
        );
    }

    #[test]
    fn a_method_of_a_trait_implementation_names_trait_and_method() {
        assert_eq!(
            parse(&format!(
                "{HEAD}detail/impl#[Store][CodeIndex]layer_edges()."
            )),
            Some(member(ty(&[], false, "CodeIndex"), "layer_edges"))
        );
    }

    #[test]
    fn other_symbols_and_unknown_forms_are_not_impl_symbols() {
        for text in [
            "rust-analyzer cargo c 0.1.0 detail/Store#save().",
            "rust-analyzer cargo c 0.1.0 impl#[Store]save().",
            "rust-analyzer cargo c 0.1.0 impl#[Vec<u8>]",
            "rust-analyzer cargo c 0.1.0 impl#[`[T; N]`]",
            "rust-analyzer cargo c 0.1.0 impl#[`dyn Any + Send`]",
            "rust-analyzer cargo c 0.1.0 impl#[&Store]",
            "rust-analyzer cargo c 0.1.0 impl#[`super::Store`]",
            "rust-analyzer cargo c 0.1.0 impl#[`self::Store`]",
            "rust-analyzer cargo c 0.1.0 impl#[`a::::Store`]",
            "rust-analyzer cargo c 0.1.0 impl#[Store",
            "rust-analyzer cargo c 0.1.0 impl#[ä",
            "rust-analyzer cargo c 0.1.0 impl#[`",
            "rust-analyzer cargo c 0.1.0 impl#[]",
            "rust-analyzer cargo c 0.1.0 impl#",
            "rust-analyzer cargo c 0.1.0 xxximpl#[Store]",
            "rust-analyzer cargo impl#[X] 0.1.0 detail/Store#save().",
            "local 3",
            "local impl#[Store]",
        ] {
            assert_eq!(parse(text), None, "{text:.60}");
        }
    }
}
