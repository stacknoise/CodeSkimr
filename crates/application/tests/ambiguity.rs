//! How many places level 1 could not narrow down (`RES-301`).

#[cfg(test)]
mod tests {
    use codeskimr_application::resolve::ambiguous_occurrences;
    use codeskimr_domain::{
        Confidence, FileId, Position, Reference, ReferenceId, ReferenceKind, Span, SymbolId,
    };

    /// A call in file 1 at `line` of the symbol `to`.
    fn call(id: u64, line: u32, to: u64, confidence: Confidence, verified: bool) -> Reference {
        let at = Position { line, column: 0 };
        Reference {
            id: ReferenceId::new(id),
            from: SymbolId::new(1),
            to: SymbolId::new(to),
            file: FileId::new(1),
            span: Span::new(at, at).expect("span"),
            kind: ReferenceKind::Call,
            confidence,
            verified,
        }
    }

    #[test]
    fn a_place_with_several_unchecked_targets_counts_once() {
        let found = [
            call(1, 5, 10, Confidence::Probable, false),
            call(2, 5, 11, Confidence::Probable, false),
            call(3, 5, 12, Confidence::Probable, false),
            call(4, 9, 10, Confidence::Probable, false),
            call(5, 9, 11, Confidence::Probable, false),
        ];
        assert_eq!(ambiguous_occurrences(found.iter()), 2);
    }

    #[test]
    fn single_targets_and_checked_references_are_not_ambiguous() {
        let found = [
            // One probable target only (a global name): not several.
            call(1, 1, 10, Confidence::Probable, false),
            call(2, 2, 10, Confidence::Certain, false),
            // Two targets that level 2 checked.
            call(3, 3, 10, Confidence::Probable, true),
            call(4, 3, 11, Confidence::Probable, true),
        ];
        assert_eq!(ambiguous_occurrences(found.iter()), 0);
    }

    #[test]
    fn targets_of_different_kinds_in_one_place_are_one_place() {
        let mut type_use = call(2, 5, 11, Confidence::Probable, false);
        type_use.kind = ReferenceKind::TypeUse;
        let found = [call(1, 5, 10, Confidence::Probable, false), type_use];
        assert_eq!(ambiguous_occurrences(found.iter()), 1);
    }
}
