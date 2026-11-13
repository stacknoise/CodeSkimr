//! Tests of the pure metric functions.

use codeskimr_domain::{
    HalsteadCounts, LineKind, Loc, cyclomatic_complexity, lines_of_code, maintainability_index,
};

const EPSILON: f64 = 1e-9;

fn counts(distinct: u32, total: u32) -> HalsteadCounts {
    HalsteadCounts {
        distinct_operators: distinct,
        distinct_operands: distinct,
        total_operators: total,
        total_operands: total,
    }
}

#[test]
fn loc_skips_blank_and_comment_lines() {
    let lines = [
        LineKind::Code,
        LineKind::Blank,
        LineKind::Comment,
        LineKind::Code,
    ];
    assert_eq!(lines_of_code(lines), Loc(2));
    assert_eq!(lines_of_code([]), Loc(0));
}

#[test]
fn complexity_starts_at_one_and_saturates() {
    assert_eq!(cyclomatic_complexity(0), 1);
    assert_eq!(cyclomatic_complexity(4), 5);
    assert_eq!(cyclomatic_complexity(u32::MAX), u32::MAX);
}

#[test]
fn volume_is_length_times_log2_of_vocabulary() {
    // N = 16, n = 8, V = 16 * 3 = 48
    assert!((counts(4, 8).volume() - 48.0).abs() < EPSILON);
    assert!(HalsteadCounts::default().volume().abs() < EPSILON);
}

#[test]
fn index_matches_the_reference_formula() {
    let value = maintainability_index(counts(4, 8), 3, Loc(10))
        .unwrap()
        .get();
    let expected =
        (171.0 - 5.2 * 48.0_f64.ln() - 0.23 * 3.0 - 16.2 * 10.0_f64.ln()) * 100.0 / 171.0;
    assert!((value - expected).abs() < EPSILON);
}

#[test]
fn index_is_clamped_to_the_scale() {
    let empty = maintainability_index(HalsteadCounts::default(), 1, Loc(0)).unwrap();
    assert!((empty.get() - 100.0).abs() < EPSILON);
    let huge = maintainability_index(counts(5000, u32::MAX / 2), u32::MAX, Loc(u32::MAX)).unwrap();
    assert!(huge.get().abs() < EPSILON);
    let tiny = maintainability_index(HalsteadCounts::default(), 1, Loc(1))
        .unwrap()
        .get();
    assert!(tiny > 99.0 && tiny <= 100.0);
}

#[test]
fn index_falls_with_size_and_complexity() {
    let small = maintainability_index(counts(4, 8), 2, Loc(10))
        .unwrap()
        .get();
    let bigger = maintainability_index(counts(4, 8), 2, Loc(200))
        .unwrap()
        .get();
    let branchy = maintainability_index(counts(4, 8), 30, Loc(10))
        .unwrap()
        .get();
    assert!(bigger < small && branchy < small);
}
