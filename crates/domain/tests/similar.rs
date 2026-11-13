//! Tests of the similar-pair search over fingerprinted bodies.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{FingerprintParams, Fingerprints, TokenId, similar_pairs, winnow};

    fn body(range: std::ops::Range<u32>) -> Fingerprints {
        let tokens: Vec<TokenId> = range
            .map(|n| TokenId::node_kind(&format!("kind{n}")))
            .collect();
        winnow(&tokens, FingerprintParams::new(3, 2).expect("valid"))
    }

    #[test]
    fn identical_bodies_form_a_pair_with_full_similarity() {
        let (a, b) = (body(0..40), body(0..40));
        let pairs = similar_pairs(&[&a, &b], 90);
        assert_eq!(pairs.len(), 1);
        assert_eq!((pairs[0].first, pairs[0].second), (0, 1));
        assert!((pairs[0].similarity - 1.0).abs() < f64::EPSILON);
        assert_eq!(pairs[0].percent, 100);
    }

    #[test]
    fn unrelated_bodies_form_no_pair() {
        let (a, b) = (body(0..40), body(100..140));
        assert_eq!(similar_pairs(&[&a, &b], 10), Vec::new());
    }

    #[test]
    fn the_value_equals_the_pairwise_similarity() {
        let (a, b, c) = (body(0..40), body(10..50), body(5..45));
        let pairs = similar_pairs(&[&a, &b, &c], 0);
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
        assert_eq!(similar_pairs(&[&a, &b, &empty, &empty], 0).len(), 1);
        assert_eq!(similar_pairs(&[&a, &b], 99), Vec::new());
    }
}
