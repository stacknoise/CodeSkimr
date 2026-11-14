//! Tests of the similar-pair search over fingerprinted bodies.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{FingerprintParams, Fingerprints, TokenId, similar_pairs, winnow};

    /// No limit on how many bodies may share a hash.
    const ALL: usize = usize::MAX;

    fn body(range: std::ops::Range<u32>) -> Fingerprints {
        let tokens: Vec<TokenId> = range
            .map(|n| TokenId::node_kind(&format!("kind{n}")))
            .collect();
        winnow(&tokens, FingerprintParams::new(3, 2).expect("valid"))
    }

    #[test]
    fn identical_bodies_form_a_pair_with_full_similarity() {
        let (a, b) = (body(0..40), body(0..40));
        let pairs = similar_pairs(&[&a, &b], 90, ALL);
        assert_eq!(pairs.len(), 1);
        assert_eq!((pairs[0].first, pairs[0].second), (0, 1));
        assert!((pairs[0].similarity - 1.0).abs() < f64::EPSILON);
        assert_eq!(pairs[0].percent, 100);
    }

    #[test]
    fn unrelated_bodies_form_no_pair() {
        let (a, b) = (body(0..40), body(100..140));
        assert_eq!(similar_pairs(&[&a, &b], 10, ALL), Vec::new());
    }

    #[test]
    fn the_value_equals_the_pairwise_similarity() {
        let (a, b, c) = (body(0..40), body(10..50), body(5..45));
        let pairs = similar_pairs(&[&a, &b, &c], 0, ALL);
        let all = [&a, &b, &c];
        for pair in &pairs {
            let expected = all[pair.first].similarity(all[pair.second]);
            assert!((pair.similarity - expected).abs() < 1e-12);
        }
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn the_minimum_filters_and_empty_bodies_never_match() {
        let (a, b) = (body(0..40), body(20..60));
        let empty = Fingerprints::default();
        assert_eq!(similar_pairs(&[&a, &b, &empty, &empty], 0, ALL).len(), 1);
        assert_eq!(similar_pairs(&[&a, &b], 99, ALL), Vec::new());
    }

    #[test]
    fn a_hash_shared_by_more_than_the_limit_nominates_no_pair() {
        // Four identical bodies share every hash. With a limit of three nothing nominates them.
        let bodies: Vec<Fingerprints> = (0..4).map(|_| body(0..40)).collect();
        let refs: Vec<&Fingerprints> = bodies.iter().collect();
        assert_eq!(similar_pairs(&refs, 90, ALL).len(), 6);
        assert_eq!(similar_pairs(&refs, 90, 3), Vec::new());
        assert_eq!(similar_pairs(&refs, 90, 4).len(), 6);
    }

    #[test]
    fn a_pair_that_a_rare_hash_nominates_is_measured_over_all_hashes() {
        // a and b share 60 tokens, c and d only the first 20 of them, which all four share. With a
        // limit of two the shared first part nominates nothing: a and b are found through the
        // hashes only they have, and their similarity is the one over all hashes. c and d have
        // nothing but shared hashes and are not found.
        let (a, b) = (body(0..60), body(0..60));
        let (c, d) = (body(0..20), body(0..20));
        let all = similar_pairs(&[&a, &b, &c, &d], 0, ALL);
        let limited = similar_pairs(&[&a, &b, &c, &d], 0, 2);
        let percent = |pairs: &[codeskimr_domain::SimilarPair], x, y| {
            pairs
                .iter()
                .find(|p| (p.first, p.second) == (x, y))
                .map(|p| p.percent)
        };
        assert_eq!(percent(&limited, 0, 1), Some(100));
        assert_eq!(percent(&limited, 0, 1), percent(&all, 0, 1));
        assert_eq!(percent(&limited, 2, 3), None);
        assert!(percent(&all, 2, 3).is_some());
    }
}
