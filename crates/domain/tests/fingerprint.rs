//! Tests of the winnowing implementation.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{FingerprintParams, Fingerprints, ModelError, TokenId, winnow};

    fn tokens(kinds: &[&str]) -> Vec<TokenId> {
        kinds.iter().map(|kind| TokenId::node_kind(kind)).collect()
    }

    fn numbered(range: std::ops::Range<u32>) -> Vec<TokenId> {
        range
            .map(|n| TokenId::node_kind(&format!("kind{n}")))
            .collect()
    }

    fn params(ngram: usize, window: usize) -> FingerprintParams {
        FingerprintParams::new(ngram, window).expect("valid parameters")
    }

    #[test]
    fn zero_parameters_are_rejected() {
        assert_eq!(FingerprintParams::new(0, 4), Err(ModelError::NotPositive));
        assert_eq!(FingerprintParams::new(4, 0), Err(ModelError::NotPositive));
    }

    #[test]
    fn identifiers_and_literals_have_fixed_distinct_tokens() {
        assert_eq!(TokenId::identifier(), TokenId::identifier());
        assert_ne!(TokenId::identifier(), TokenId::literal());
        assert_ne!(TokenId::identifier(), TokenId::node_kind("if"));
    }

    #[test]
    fn hashes_are_stable() {
        let stream = tokens(&["a", "b", "c", "d", "e", "f"]);
        let first = winnow(&stream, params(2, 3));
        let second = winnow(&stream, params(2, 3));
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn short_streams_yield_nothing_or_one_window() {
        let stream = tokens(&["a", "b", "c"]);
        assert!(winnow(&stream, params(4, 2)).is_empty());
        assert_eq!(winnow(&stream, params(3, 5)).entries().len(), 1);
        assert!(winnow(&[], params(2, 2)).is_empty());
    }

    #[test]
    fn window_of_one_keeps_every_ngram() {
        let stream = numbered(0..10);
        assert_eq!(winnow(&stream, params(3, 1)).entries().len(), 8);
    }

    #[test]
    fn positions_ascend_and_are_unique() {
        let stream = numbered(0..200);
        let prints = winnow(&stream, params(4, 8));
        let positions: Vec<u32> = prints.entries().iter().map(|e| e.position).collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(positions.len() < 200 - 3);
    }

    #[test]
    fn shared_run_of_guaranteed_length_shares_a_fingerprint() {
        let (ngram, window) = (3, 5);
        let run = numbered(100..100 + 7);
        assert_eq!(run.len(), window + ngram - 1);
        let mut left = numbered(0..20);
        left.extend(run.clone());
        left.extend(numbered(200..210));
        let mut right = numbered(300..315);
        right.extend(run);
        right.extend(numbered(400..403));
        let a = winnow(&left, params(ngram, window));
        let b = winnow(&right, params(ngram, window));
        let shared = a
            .entries()
            .iter()
            .any(|x| b.entries().iter().any(|y| x.hash == y.hash));
        assert!(shared);
    }

    #[test]
    fn identical_streams_are_fully_similar_and_unrelated_ones_are_not() {
        let stream = numbered(0..60);
        let other = numbered(1000..1060);
        let a = winnow(&stream, params(4, 4));
        let b = winnow(&other, params(4, 4));
        assert!((a.similarity(&a) - 1.0).abs() < f64::EPSILON);
        assert!(a.similarity(&b) < f64::EPSILON);
        assert!(a.similarity(&Fingerprints::default()) < f64::EPSILON);
    }

    #[test]
    fn containment_finds_a_snippet_inside_a_longer_body() {
        let body = numbered(0..120);
        let snippet: Vec<TokenId> = body.iter().skip(30).take(40).copied().collect();
        let whole = winnow(&body, params(3, 4));
        let part = winnow(&snippet, params(3, 4));
        assert!(part.containment(&whole) > 0.8);
        assert!(whole.containment(&part) < 0.8);
    }
}
