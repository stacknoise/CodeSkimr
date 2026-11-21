//! The parameters of a signature as written, the key that tells overloads apart (ADR 0068).

use crate::consts::{
    ARROW_HEADS, ARROW_TAIL, CLOSERS, OPENERS, PARAMETER_LIST_END, PARAMETER_LIST_START,
    PARAMETER_SEPARATOR,
};

/// The parameters of the first parameter list of `signature` as written, in order, each trimmed
/// (`int Total(int a, Map<K, V> b)` gives `int a` and `Map<K, V> b`). Commas inside brackets or
/// angle brackets do not split. A signature without a parameter list, or an empty one, gives none.
#[must_use]
pub fn signature_parameters(signature: &str) -> Vec<String> {
    let Some(start) = signature.find(PARAMETER_LIST_START) else {
        return Vec::new();
    };
    let list = signature
        .get(start + PARAMETER_LIST_START.len_utf8()..)
        .unwrap_or_default();
    let mut parameters = Vec::new();
    let mut depth = 0_usize;
    let mut from = 0;
    let mut previous = None;
    for (at, character) in list.char_indices() {
        let closes_list = depth == 0 && character == PARAMETER_LIST_END;
        if closes_list || (depth == 0 && character == PARAMETER_SEPARATOR) {
            parameters.push(list.get(from..at).unwrap_or_default().trim());
            from = at + character.len_utf8();
        }
        if closes_list {
            break;
        }
        if OPENERS.contains(&character) {
            depth += 1;
        } else if CLOSERS.contains(&character)
            && !(character == ARROW_TAIL
                && previous.is_some_and(|head| ARROW_HEADS.contains(&head)))
        {
            depth = depth.saturating_sub(1);
        }
        previous = Some(character);
    }
    parameters
        .into_iter()
        .filter(|parameter| !parameter.is_empty())
        .map(str::to_owned)
        .collect()
}
