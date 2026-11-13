//! `CHK-104`: functions with more code lines than allowed (R-02).

use std::collections::HashMap;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{RelativePath, SymbolCounts, SymbolId};

use super::model::{Detail, Finding, Location};
use crate::ports::FileFacts;

/// A function-like symbol with a body, as the checks see it.
pub(super) struct Callable {
    pub location: Location,
    pub lines: u32,
    pub id: SymbolId,
}

/// The callables of a file that have a measured body.
pub(super) fn callables(path: &RelativePath, facts: &FileFacts) -> Vec<Callable> {
    let counts: HashMap<SymbolId, &SymbolCounts> = facts
        .counts
        .iter()
        .map(|count| (count.symbol, count))
        .collect();
    facts
        .symbols
        .iter()
        .filter(|symbol| symbol.kind.is_callable())
        .filter_map(|symbol| {
            let count = counts.get(&symbol.id)?;
            count.body?;
            Some(Callable {
                location: Location {
                    path: path.clone(),
                    span: symbol.span,
                    symbol: symbol.qualified_name.clone(),
                },
                lines: count.loc.0,
                id: symbol.id,
            })
        })
        .collect()
}

/// One finding for every callable longer than `limit`.
pub(super) fn too_long(callables: &[Callable], limit: u32) -> Vec<Finding> {
    callables
        .iter()
        .filter(|callable| callable.lines > limit)
        .map(|callable| Finding {
            code: ErrorCode::FunctionTooLong,
            location: callable.location.clone(),
            detail: Detail::TooLong {
                lines: callable.lines,
                limit,
            },
        })
        .collect()
}
