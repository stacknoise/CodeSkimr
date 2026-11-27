//! The metric changes between two scans (ADR 0114).

#[cfg(test)]
mod tests {
    use codeskimr_domain::{
        Crossing, MetricFact, MetricKind, MetricLevel, MetricValues, MoveDirection, MoveLimits,
        QualifiedName, RelativePath, SymbolKind, Verdict, compare_metrics,
    };

    fn limits() -> MoveLimits {
        MoveLimits {
            cc: 2.0,
            mi: 5.0,
            ce: 2.0,
            instability: 0.1,
            loc_growth: 20.0,
            busy_dependents: 10.0,
            poor_cc_above: 10.0,
            poor_mi_below: 35.0,
            poor_instability_above: 0.8,
        }
    }

    fn fact(kind: SymbolKind, name: &str, values: MetricValues) -> MetricFact {
        MetricFact {
            kind,
            qualified_name: QualifiedName::new(name).unwrap(),
            signature: Some("(a: i32)".to_owned()),
            path: RelativePath::new("src/a.rs").unwrap(),
            values,
        }
    }

    fn member(cc: f64, mi: f64, loc: f64) -> MetricValues {
        MetricValues {
            cc: Some(cc),
            mi: Some(mi),
            loc: Some(loc),
            ..MetricValues::default()
        }
    }

    fn one(base: MetricValues, target: MetricValues) -> Vec<codeskimr_domain::MetricChange> {
        let kind = SymbolKind::Method;
        compare_metrics(
            &[fact(kind, "a::f", base)],
            &[fact(kind, "a::f", target)],
            &limits(),
        )
    }

    #[test]
    fn a_move_below_the_threshold_is_not_listed() {
        assert_eq!(
            one(member(6.0, 71.0, 22.0), member(7.0, 68.0, 30.0)),
            Vec::new()
        );
    }

    #[test]
    fn more_complexity_and_a_lower_index_are_worse_and_growth_is_only_marked() {
        let found = one(member(6.0, 71.0, 22.0), member(14.0, 58.0, 47.0));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].verdict, Verdict::Worse);
        let kinds: Vec<_> = found[0]
            .moves
            .iter()
            .map(|m| (m.kind, m.direction))
            .collect();
        assert!(kinds.contains(&(MetricKind::CyclomaticComplexity, MoveDirection::Worse)));
        assert!(kinds.contains(&(MetricKind::MaintainabilityIndex, MoveDirection::Worse)));
        assert!(kinds.contains(&(MetricKind::Loc, MoveDirection::Grew)));
        assert!(found[0].crosses_on(MoveDirection::Worse));
    }

    #[test]
    fn lines_alone_are_never_a_verdict() {
        assert_eq!(
            one(member(6.0, 71.0, 22.0), member(6.0, 71.0, 90.0)),
            Vec::new()
        );
    }

    #[test]
    fn a_move_to_both_sides_is_mixed() {
        let found = one(member(6.0, 50.0, 22.0), member(12.0, 70.0, 22.0));
        assert_eq!(found[0].verdict, Verdict::Mixed);
    }

    #[test]
    fn leaving_the_poor_range_is_marked() {
        let found = one(member(18.0, 49.0, 88.0), member(9.0, 61.0, 54.0));
        assert_eq!(found[0].verdict, Verdict::Better);
        let cc = found[0]
            .moves
            .iter()
            .find(|m| m.kind == MetricKind::CyclomaticComplexity);
        assert_eq!(cc.unwrap().crossing, Some(Crossing::OutOfPoor));
    }

    #[test]
    fn a_rising_instability_is_worse_only_for_a_module_that_many_use() {
        let module = |instability, ca| MetricValues {
            instability: Some(instability),
            ca: Some(ca),
            ..MetricValues::default()
        };
        let compare = |ca| {
            compare_metrics(
                &[fact(SymbolKind::Module, "m", module(0.42, ca))],
                &[fact(SymbolKind::Module, "m", module(0.61, ca))],
                &limits(),
            )
        };
        assert_eq!(compare(14.0)[0].verdict, Verdict::Worse);
        assert_eq!(compare(3.0), Vec::new());
    }

    #[test]
    fn an_object_in_one_scan_only_is_not_compared() {
        let kind = SymbolKind::Method;
        let found = compare_metrics(
            &[fact(kind, "a::old", member(1.0, 80.0, 5.0))],
            &[fact(kind, "a::new", member(30.0, 20.0, 99.0))],
            &limits(),
        );
        assert_eq!(found, Vec::new());
    }

    #[test]
    fn the_level_follows_the_kind() {
        assert_eq!(
            MetricLevel::of(SymbolKind::Namespace),
            Some(MetricLevel::Module)
        );
        assert_eq!(MetricLevel::of(SymbolKind::Class), Some(MetricLevel::Type));
        assert_eq!(
            MetricLevel::of(SymbolKind::Method),
            Some(MetricLevel::Member)
        );
        assert_eq!(MetricLevel::of(SymbolKind::Parameter), None);
    }

    #[test]
    fn a_crossing_counts_only_on_the_side_it_moved_to() {
        let found = one(member(9.0, 30.0, 22.0), member(14.0, 45.0, 22.0));
        assert_eq!(found[0].verdict, Verdict::Mixed);
        assert!(
            found[0].crosses_on(MoveDirection::Worse),
            "complexity entered the poor range"
        );
        assert!(
            found[0].crosses_on(MoveDirection::Better),
            "the index left the poor range"
        );
        let found = one(member(6.0, 50.0, 22.0), member(12.0, 70.0, 22.0));
        assert!(found[0].crosses_on(MoveDirection::Worse));
        assert!(
            !found[0].crosses_on(MoveDirection::Better),
            "70 was not poor before"
        );
    }

    #[test]
    fn a_move_of_exactly_the_threshold_counts_whatever_the_rounding() {
        let module = |instability| MetricValues {
            instability: Some(instability),
            ca: Some(14.0),
            ..MetricValues::default()
        };
        let found = compare_metrics(
            &[fact(SymbolKind::Module, "m", module(0.5))],
            &[fact(SymbolKind::Module, "m", module(0.6))],
            &limits(),
        );
        assert_eq!(
            found.len(),
            1,
            "0.6 - 0.5 is a hair under 0.1 in floating point"
        );
    }
}
