//! The judgement of the unused symbols under the exception list (ADR 0101).

#[cfg(test)]
mod tests {
    use codeskimr_application::config::UnusedIgnore;
    use codeskimr_domain::{
        LanguageId, Measures, QualifiedName, RelativePath, SymbolFacts, SymbolId, SymbolKind,
        SymbolName, UsageTally, Visibility,
    };

    use codeskimr_application::unused::{Judgement, judge};

    fn fact(id: u64, path: &str, name: &str, visibility: Option<Visibility>) -> SymbolFacts {
        SymbolFacts {
            id: SymbolId::new(id),
            name: SymbolName::new(name).unwrap(),
            qualified_name: QualifiedName::new(name).unwrap(),
            kind: SymbolKind::Function,
            parent: None,
            visibility,
            path: RelativePath::new(path).unwrap(),
            language: LanguageId::new("csharp").unwrap(),
            test: false,
            overrides: false,
            usage: UsageTally::default(),
            measures: Measures::default(),
        }
    }

    fn entry(paths: &[&str], symbols: &[&str]) -> UnusedIgnore {
        UnusedIgnore {
            paths: paths.iter().map(ToString::to_string).collect(),
            symbols: symbols.iter().map(ToString::to_string).collect(),
            reason: "on purpose".to_owned(),
        }
    }

    fn ids(facts: &[SymbolFacts]) -> Vec<u64> {
        facts.iter().map(|fact| fact.id.get()).collect()
    }

    #[test]
    fn the_judgement_keeps_the_places_inside_an_assessment_and_names_the_first_covering_entry() {
        let rules = [entry(&[], &["Hidden*"]), entry(&["gen/*"], &[])];
        let mut test_fact = fact(5, "tests/a.cs", "Check", Some(Visibility::Private));
        test_fact.test = true;
        let unused = vec![
            fact(1, "a.cs", "Open", Some(Visibility::Public)),
            fact(2, "a.cs", "Closed", Some(Visibility::Private)),
            fact(3, "b.cs", "HiddenOne", Some(Visibility::Private)),
            fact(4, "gen/c.cs", "Made", Some(Visibility::Private)),
            test_fact,
            fact(6, "d.cs", "Later", Some(Visibility::Private)),
        ];
        let judged = judge(
            unused.clone(),
            &Judgement {
                include_tests: false,
                rules: &rules,
                approximate: &[],
            },
        );
        // The very likely unused ones first, each group in the order of the places it came in.
        assert_eq!(ids(&judged.kept), vec![2, 6, 1]);
        let left: Vec<(u64, usize)> = judged
            .left_out
            .iter()
            .map(|(fact, rule)| (fact.id.get(), *rule))
            .collect();
        assert_eq!(left, vec![(3, 0), (4, 1)]);
        assert_eq!(judged.tests_hidden, 1);

        let with_tests = judge(
            unused,
            &Judgement {
                include_tests: true,
                rules: &rules,
                approximate: &["csharp".to_owned()],
            },
        );
        // A language of approximate usages puts everything into the last group.
        assert_eq!(ids(&with_tests.kept), vec![1, 2, 5, 6]);
        assert_eq!(with_tests.tests_hidden, 0);
    }
}
