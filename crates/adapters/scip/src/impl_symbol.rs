//! The symbols rust-analyzer gives to impl blocks. They are no descriptors of a type the project
//! has a definition for, so they are read apart: `impl#[Type]` is what `Self` names inside the
//! block, and a member of `impl#[Type][Trait]` is the implementation of a method of the trait
//! (ADR 0033, decision 2).

use crate::consts::{
    CLOSE_BRACKET, IMPL_MARKER, OPEN_BRACKET, OPEN_GENERIC, PATH_SEPARATOR, SUFFIX_NAMESPACE,
    SUFFIX_TYPE, SYMBOL_SEPARATOR,
};
use crate::symbol::{Chars, DescriptorKind, is_simple, name, named};

/// What an impl symbol stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImplSymbol {
    /// The impl block itself, as `Self` names it. `prefix` is the symbol text before `impl#`, so
    /// the symbol of the type in the same module is `prefix` + name + `#`.
    SelfType {
        /// The symbol text up to `impl#`.
        prefix: String,
        /// The name of the type the block is for.
        name: String,
    },
    /// A method in a trait implementation.
    TraitMember {
        /// The name of the trait.
        trait_name: String,
        /// The name of the method.
        member: String,
    },
}

/// The bare name of a type or trait as rust-analyzer writes it inside the brackets: generic
/// arguments are cut off (`Job<Queued>` is `Job`) and so is a path (`crate::Repository`). `None`
/// if what remains is not a plain name (`&Type`, `[T; N]`, `dyn Any`).
fn bare_name(text: &str) -> Option<String> {
    let base = text.split(OPEN_GENERIC).next()?;
    let last = base.rsplit(PATH_SEPARATOR).next()?;
    (!last.is_empty() && last.chars().all(is_simple)).then(|| last.to_owned())
}

/// `[name]`; the opening bracket is already read. The name is quoted if it is not simple.
fn bracket_name(chars: &mut Chars<'_>) -> Option<String> {
    let text = name(chars)?;
    (chars.next()? == CLOSE_BRACKET)
        .then(|| bare_name(&text))
        .flatten()
}

/// Reads the impl symbol `text`, or `None` if it is not one or has a form the import does not
/// know (a generic type written with its arguments, a reference type).
pub(crate) fn parse(text: &str) -> Option<ImplSymbol> {
    let at = text.find(IMPL_MARKER)?;
    let prefix = text.get(..at)?;
    if !prefix.ends_with([SYMBOL_SEPARATOR, SUFFIX_NAMESPACE, SUFFIX_TYPE]) {
        return None;
    }
    let mut chars = text.get(at + IMPL_MARKER.len()..)?.chars().peekable();
    let mut names = vec![bracket_name(&mut chars)?];
    while chars.next_if_eq(&OPEN_BRACKET).is_some() {
        names.push(bracket_name(&mut chars)?);
    }
    let mut names = names.into_iter();
    let (first, second) = (names.next()?, names.next());
    if chars.peek().is_none() {
        return Some(ImplSymbol::SelfType {
            prefix: prefix.to_owned(),
            name: first,
        });
    }
    let member = named(&mut chars)?;
    (chars.peek().is_none() && member.kind == DescriptorKind::Method).then_some(
        ImplSymbol::TraitMember {
            trait_name: second?,
            member: member.name,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn self_type(prefix: &str, name: &str) -> ImplSymbol {
        ImplSymbol::SelfType {
            prefix: prefix.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn the_block_of_a_type_and_of_a_trait_implementation_name_the_type() {
        let head = "rust-analyzer cargo codeskimr_domain 0.1.0 ";
        assert_eq!(
            parse(&format!("{head}fingerprint/impl#[Fingerprints]")),
            Some(self_type(&format!("{head}fingerprint/"), "Fingerprints"))
        );
        assert_eq!(
            parse(&format!("{head}impl#[Store][CodeIndex]")),
            Some(self_type(head, "Store"))
        );
    }

    #[test]
    fn generic_arguments_and_paths_are_cut_off_the_names() {
        let head = "rust-analyzer cargo c 0.1.0 ";
        assert_eq!(
            parse(&format!("{head}impl#[`Job<Queued>`]")),
            Some(self_type(head, "Job"))
        );
        assert_eq!(
            parse(&format!("{head}impl#[`crate::Repository`]")),
            Some(self_type(head, "Repository"))
        );
        assert_eq!(
            parse(&format!("{head}impl#[CodedError][`From<StorageError>`]")),
            Some(self_type(head, "CodedError"))
        );
        assert_eq!(
            parse(&format!("{head}impl#[Store][`Index<I>`]get().")),
            Some(ImplSymbol::TraitMember {
                trait_name: "Index".to_owned(),
                member: "get".to_owned(),
            })
        );
    }

    #[test]
    fn a_method_of_a_trait_implementation_names_trait_and_method() {
        let text = "rust-analyzer cargo c 0.1.0 detail/impl#[Store][CodeIndex]layer_edges().";
        assert_eq!(
            parse(text),
            Some(ImplSymbol::TraitMember {
                trait_name: "CodeIndex".to_owned(),
                member: "layer_edges".to_owned(),
            })
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
            "rust-analyzer cargo c 0.1.0 impl#[Store",
            "rust-analyzer cargo c 0.1.0 xxximpl#[Store]",
            "local 3",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
