//! The reverse index: direct and transitive usages, cycles, tests, order.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{
        Confidence, Direction, FileId, Position, Reached, Reference, ReferenceId, ReferenceKind,
        ReverseIndex, Span, SymbolId, UsageQuery,
    };

    fn reference(id: u64, from: u64, to: u64, kind: ReferenceKind) -> Reference {
        let position = Position {
            line: u32::try_from(id).unwrap(),
            column: 0,
        };
        Reference {
            id: ReferenceId::new(id),
            from: SymbolId::new(from),
            to: SymbolId::new(to),
            file: FileId::new(1),
            span: Span::new(position, position).unwrap(),
            kind,
            confidence: Confidence::Certain,
        }
    }

    fn call(id: u64, from: u64, to: u64) -> Reference {
        reference(id, from, to, ReferenceKind::Call)
    }

    fn query(symbol: u64, direction: Direction, depth: u32) -> UsageQuery {
        UsageQuery {
            symbol: SymbolId::new(symbol),
            direction,
            depth,
            include_tests: true,
        }
    }

    fn ids(index: &ReverseIndex, query: &UsageQuery) -> Vec<(u32, u64)> {
        index
            .usages(query)
            .iter()
            .map(|usage| (usage.depth, usage.reference.id.get()))
            .collect()
    }

    /// 1 -> 2 -> 3 -> 4, and 5 -> 3.
    fn chain() -> ReverseIndex {
        ReverseIndex::new([
            call(10, 1, 2),
            call(11, 2, 3),
            call(12, 3, 4),
            call(13, 5, 3),
        ])
    }

    #[test]
    fn incoming_depth_one_lists_the_direct_references_only() {
        let found = ids(&chain(), &query(3, Direction::Incoming, 1));
        assert_eq!(found, [(1, 11), (1, 13)]);
    }

    #[test]
    fn incoming_depth_two_adds_the_users_of_the_users() {
        let found = ids(&chain(), &query(3, Direction::Incoming, 2));
        assert_eq!(found, [(1, 11), (1, 13), (2, 10)]);
    }

    #[test]
    fn outgoing_follows_what_the_symbol_uses() {
        let found = ids(&chain(), &query(2, Direction::Outgoing, 5));
        assert_eq!(found, [(1, 11), (2, 12)]);
    }

    #[test]
    fn cycles_end_and_a_reference_shows_up_once() {
        let index = ReverseIndex::new([call(10, 1, 2), call(11, 2, 1), call(12, 1, 1)]);
        let found = ids(&index, &query(1, Direction::Incoming, 9));
        assert_eq!(found, [(1, 11), (1, 12), (2, 10)]);
    }

    #[test]
    fn tests_count_only_when_asked() {
        let index = ReverseIndex::new([call(10, 1, 3), reference(11, 2, 3, ReferenceKind::Test)]);
        let mut question = query(3, Direction::Incoming, 1);
        assert_eq!(ids(&index, &question), [(1, 10), (1, 11)]);
        question.include_tests = false;
        assert_eq!(ids(&index, &question), [(1, 10)]);
    }

    #[test]
    fn reached_lists_each_symbol_once_nearest_first() {
        let index = ReverseIndex::new([
            call(10, 1, 2),
            call(11, 3, 2),
            call(12, 1, 3),
            call(13, 4, 1),
        ]);
        let reached = index.reached(&query(2, Direction::Incoming, 5));
        let ordered: Vec<(u64, u32)> = reached
            .iter()
            .map(|Reached { symbol, depth }| (symbol.get(), *depth))
            .collect();
        assert_eq!(ordered, [(1, 1), (3, 1), (4, 2)]);
    }

    #[test]
    fn zero_depth_and_unknown_symbols_find_nothing() {
        let index = chain();
        assert_eq!(index.usages(&query(3, Direction::Incoming, 0)).len(), 0);
        assert_eq!(index.usages(&query(99, Direction::Incoming, 3)).len(), 0);
    }

    #[test]
    fn counts_and_order_do_not_depend_on_the_input_order() {
        let forward = ReverseIndex::new([call(10, 1, 3), call(11, 2, 3)]);
        let backward = ReverseIndex::new([call(11, 2, 3), call(10, 1, 3)]);
        let question = query(3, Direction::Incoming, 1);
        assert_eq!(forward.usages(&question), backward.usages(&question));
        assert_eq!(forward.usage_count(SymbolId::new(3)), 2);
        assert_eq!(forward.len(), 2);
        assert_eq!(ReverseIndex::default().len(), 0);
    }

    #[test]
    fn a_symbol_reached_twice_is_listed_once_and_both_references_show() {
        let index = ReverseIndex::new([
            call(10, 2, 1),
            call(11, 3, 1),
            call(12, 4, 2),
            call(13, 4, 3),
        ]);
        let question = query(1, Direction::Incoming, 3);
        assert_eq!(ids(&index, &question), [(1, 10), (1, 11), (2, 12), (2, 13)]);
        assert_eq!(index.reached(&question).len(), 3);
    }

    #[test]
    fn tests_are_left_out_at_later_steps_too() {
        let index = ReverseIndex::new([call(10, 2, 1), reference(11, 3, 2, ReferenceKind::Test)]);
        let mut question = query(1, Direction::Incoming, 3);
        assert_eq!(ids(&index, &question), [(1, 10), (2, 11)]);
        question.include_tests = false;
        assert_eq!(ids(&index, &question), [(1, 10)]);
    }
}
