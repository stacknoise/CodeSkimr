//! The comparison of the symbols of two snapshots (ADR 0097).

#[cfg(test)]
mod tests {
    use codeskimr_domain::{
        ChangeCause, ChangeFilter, ChangeReason, ContentHash, ImpactCause, QualifiedName,
        RelativePath, SymbolFact, SymbolId, SymbolKind, Visibility, compare_symbols,
    };

    fn fact(id: u64, kind: SymbolKind, name: &str, signature: Option<&str>) -> SymbolFact {
        SymbolFact {
            id: SymbolId::new(id),
            kind,
            qualified_name: QualifiedName::new(name).unwrap(),
            signature: signature.map(str::to_owned),
            visibility: Some(Visibility::Public),
            path: RelativePath::new("src/a.rs").unwrap(),
            content_hash: Some(ContentHash::new(format!("h:{name}")).unwrap()),
        }
    }

    fn method(id: u64, name: &str, signature: &str) -> SymbolFact {
        fact(id, SymbolKind::Method, name, Some(signature))
    }

    fn with_hash(mut fact: SymbolFact, hash: &str) -> SymbolFact {
        fact.content_hash = Some(ContentHash::new(hash).unwrap());
        fact
    }

    fn in_file(mut fact: SymbolFact, path: &str) -> SymbolFact {
        fact.path = RelativePath::new(path).unwrap();
        fact
    }

    #[test]
    fn equal_snapshots_have_no_difference() {
        let symbols = vec![
            method(1, "A.f", "int f(int a)"),
            method(2, "A.g", "void g()"),
        ];
        let other: Vec<SymbolFact> = symbols
            .iter()
            .cloned()
            .map(|mut fact| {
                fact.id = SymbolId::new(fact.id.get() + 100);
                fact
            })
            .collect();
        assert!(compare_symbols(&symbols, &other).is_empty());
    }

    #[test]
    fn a_symbol_only_one_side_has_is_added_or_removed() {
        let base = vec![method(1, "A.old", "void old()")];
        let target = vec![method(2, "A.fresh", "void fresh()")];
        let found = compare_symbols(&base, &target);
        assert_eq!(found.removed.len(), 1);
        assert_eq!(found.removed[0].qualified_name.as_str(), "A.old");
        assert_eq!(found.added.len(), 1);
        assert_eq!(found.added[0].qualified_name.as_str(), "A.fresh");
        assert!(found.changed.is_empty() && found.moved.is_empty());
    }

    #[test]
    fn overloads_are_told_apart_by_their_parameter_types() {
        let base = vec![
            method(1, "A.f", "void f(int a)"),
            method(2, "A.f", "void f(string a)"),
        ];
        let target = vec![method(3, "A.f", "void f(int a)")];
        let found = compare_symbols(&base, &target);
        assert_eq!(found.removed.len(), 1);
        assert_eq!(
            found.removed[0].signature.as_deref(),
            Some("void f(string a)")
        );
        assert!(found.added.is_empty() && found.changed.is_empty());
    }

    #[test]
    fn a_changed_return_type_is_a_signature_change() {
        let base = vec![method(1, "A.f", "int f(int a)")];
        let target = vec![with_hash(method(2, "A.f", "long f(int a)"), "h1")];
        let found = compare_symbols(&base, &target);
        assert_eq!(found.changed.len(), 1);
        assert_eq!(found.changed[0].reasons, vec![ChangeReason::Signature]);
    }

    #[test]
    fn a_narrower_visibility_is_marked() {
        let base = vec![method(1, "A.f", "void f()")];
        let mut narrower = method(2, "A.f", "void f()");
        narrower.visibility = Some(Visibility::Private);
        let found = compare_symbols(&base, &[narrower]);
        assert_eq!(
            found.changed[0].reasons,
            vec![ChangeReason::Visibility { narrowed: true }]
        );
        let mut wider = method(3, "A.f", "void f()");
        wider.visibility = Some(Visibility::Public);
        let mut private = method(4, "A.f", "void f()");
        private.visibility = Some(Visibility::Private);
        let found = compare_symbols(&[private], &[wider]);
        assert_eq!(
            found.changed[0].reasons,
            vec![ChangeReason::Visibility { narrowed: false }]
        );
    }

    #[test]
    fn only_the_content_hash_makes_a_content_change() {
        let base = vec![method(1, "A.f", "void f()")];
        let target = vec![with_hash(method(2, "A.f", "void f()"), "h1")];
        let found = compare_symbols(&base, &target);
        assert_eq!(found.changed[0].reasons, vec![ChangeReason::Content]);
        assert_eq!(found.impact_subjects().len(), 0);
    }

    #[test]
    fn a_symbol_without_a_hash_changes_by_signature_or_visibility_only() {
        let mut base = method(1, "A.f", "void f()");
        base.content_hash = None;
        let target = with_hash(method(2, "A.f", "void f()"), "h9");
        assert!(compare_symbols(&[base.clone()], &[target]).is_empty());
        let mut target = method(3, "A.f", "int f()");
        target.content_hash = None;
        let found = compare_symbols(&[base], &[target]);
        assert_eq!(found.changed[0].reasons, vec![ChangeReason::Signature]);
    }

    #[test]
    fn the_same_key_in_another_file_is_a_move() {
        let base = vec![method(1, "A.f", "void f()")];
        let target = vec![in_file(method(2, "A.f", "void f()"), "src/b.rs")];
        let found = compare_symbols(&base, &target);
        assert_eq!(found.moved.len(), 1);
        assert_eq!(found.moved[0].reasons.len(), 0);
        assert!(found.added.is_empty() && found.removed.is_empty() && found.changed.is_empty());
        assert_eq!(found.impact_subjects().len(), 0);
    }

    #[test]
    fn a_renamed_symbol_is_removed_and_added_and_its_usages_are_looked_at() {
        let base = vec![with_hash(method(1, "A.old", "void old()"), "same")];
        let target = vec![with_hash(method(2, "A.fresh", "void fresh()"), "same")];
        let found = compare_symbols(&base, &target);
        assert_eq!((found.removed.len(), found.added.len()), (1, 1));
        assert_eq!(found.moved.len(), 0);
        let subjects = found.impact_subjects();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].cause, ImpactCause::Removed);
    }

    #[test]
    fn namespaces_modules_parameters_and_aliases_take_no_part() {
        let kinds = [
            SymbolKind::Namespace,
            SymbolKind::Module,
            SymbolKind::Parameter,
            SymbolKind::TypeAlias,
        ];
        for (at, kind) in kinds.into_iter().enumerate() {
            let id = u64::try_from(at).unwrap() + 1;
            let found = compare_symbols(&[], &[fact(id, kind, "A.x", None)]);
            assert!(found.is_empty(), "{kind} must not take part");
        }
        let found = compare_symbols(&[], &[fact(1, SymbolKind::EnumMember, "E.A", None)]);
        assert_eq!(found.added.len(), 1);
    }

    #[test]
    fn the_same_key_twice_pairs_by_file_and_leaves_the_rest_alone() {
        let base = vec![
            method(1, "A.f", "void f()"),
            in_file(method(2, "A.f", "void f()"), "src/b.rs"),
        ];
        let target = vec![
            in_file(method(3, "A.f", "void f()"), "src/b.rs"),
            method(4, "A.f", "void f()"),
        ];
        assert!(compare_symbols(&base, &target).is_empty());
        let three = vec![
            method(5, "A.f", "void f()"),
            in_file(method(6, "A.f", "void f()"), "src/c.rs"),
            in_file(method(7, "A.f", "void f()"), "src/d.rs"),
        ];
        let found = compare_symbols(&base[..1], &three);
        assert_eq!(found.added.len(), 2);
        assert!(found.moved.is_empty() && found.removed.is_empty());
    }

    #[test]
    fn impact_subjects_are_removed_and_signature_or_visibility_changes() {
        let base = vec![
            method(1, "A.gone", "void gone()"),
            method(2, "A.sig", "int sig()"),
            method(3, "A.body", "void body()"),
        ];
        let target = vec![
            method(12, "A.sig", "long sig()"),
            with_hash(method(13, "A.body", "void body()"), "other"),
        ];
        let found = compare_symbols(&base, &target);
        let subjects = found.impact_subjects();
        let names: Vec<&str> = subjects
            .iter()
            .map(|subject| subject.symbol.qualified_name.as_str())
            .collect();
        assert_eq!(names, vec!["A.gone", "A.sig"]);
        let touched = found.touched_in_base();
        assert_eq!(touched.len(), 3);
        assert!(touched.contains(&SymbolId::new(3)));
    }

    #[test]
    fn the_lists_are_ordered_by_file_name_and_id() {
        let target = vec![
            in_file(method(1, "B.f", "void f()"), "src/z.rs"),
            method(2, "B.g", "void g()"),
            method(3, "A.h", "void h()"),
        ];
        let found = compare_symbols(&[], &target);
        let names: Vec<&str> = found
            .added
            .iter()
            .map(|fact| fact.qualified_name.as_str())
            .collect();
        assert_eq!(names, vec!["A.h", "B.g", "B.f"]);
    }

    /// Two removed, one added, one changed and one moved symbol, of kinds and visibilities that
    /// the filter tells apart.
    fn mixed() -> codeskimr_domain::Comparison {
        let mut private = method(3, "Core.hidden", "void hidden()");
        private.visibility = Some(Visibility::Private);
        let base = vec![
            method(1, "Core.Run", "void Run()"),
            fact(2, SymbolKind::Class, "Core.Old", None),
            private,
            method(4, "Core.edit", "int edit()"),
            method(5, "Core.mv", "void mv()"),
        ];
        let target = vec![
            method(14, "Core.edit", "long edit()"),
            in_file(method(15, "Core.mv", "void mv()"), "src/b.rs"),
            method(16, "Core.Fresh", "void Fresh()"),
        ];
        compare_symbols(&base, &target)
    }

    fn names(found: &codeskimr_domain::Comparison) -> Vec<&str> {
        let removed = found
            .removed
            .iter()
            .map(|fact| fact.qualified_name.as_str());
        let added = found.added.iter().map(|fact| fact.qualified_name.as_str());
        let changed = found
            .changed
            .iter()
            .map(|it| it.target.qualified_name.as_str());
        let moved = found
            .moved
            .iter()
            .map(|it| it.target.qualified_name.as_str());
        removed.chain(added).chain(changed).chain(moved).collect()
    }

    #[test]
    fn an_empty_filter_keeps_everything() {
        let found = mixed();
        assert_eq!(found.filtered(&ChangeFilter::default()), found);
    }

    #[test]
    fn the_name_is_searched_anywhere_ignoring_case() {
        let filter = ChangeFilter {
            name: " ED ".to_owned(),
            ..ChangeFilter::default()
        };
        assert_eq!(names(&mixed().filtered(&filter)), vec!["Core.edit"]);
    }

    #[test]
    fn a_cause_keeps_only_that_list() {
        let filter = ChangeFilter {
            cause: Some(ChangeCause::Removed),
            ..ChangeFilter::default()
        };
        assert_eq!(
            names(&mixed().filtered(&filter)),
            vec!["Core.Old", "Core.Run", "Core.hidden"]
        );
    }

    #[test]
    fn kind_and_visibility_narrow_every_list() {
        let kind = ChangeFilter {
            kind: Some(SymbolKind::Class),
            ..ChangeFilter::default()
        };
        assert_eq!(names(&mixed().filtered(&kind)), vec!["Core.Old"]);
        let public = ChangeFilter {
            public_only: true,
            ..ChangeFilter::default()
        };
        assert!(!names(&mixed().filtered(&public)).contains(&"Core.hidden"));
    }

    #[test]
    fn the_whole_difference_still_tells_what_counts_as_adapted() {
        let found = mixed();
        let filter = ChangeFilter {
            name: "Run".to_owned(),
            ..ChangeFilter::default()
        };
        let kept = found.filtered(&filter);
        assert_eq!(kept.impact_subjects().len(), 1);
        assert!(found.touched_in_base().len() > kept.touched_in_base().len());
    }
}
