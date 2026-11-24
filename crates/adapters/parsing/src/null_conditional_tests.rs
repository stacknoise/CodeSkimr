use super::prepare;

#[test]
fn the_mark_of_an_assigned_member_becomes_a_space_of_the_same_width() {
    let source = "a?.B = 1; a?.B.C += 2; a?.B ??= 3; a?[0] = 4; a?.B |= 5;";
    let fixed = prepare(source);
    assert_eq!(
        fixed,
        "a .B = 1; a .B.C += 2; a .B ??= 3; a [0] = 4; a .B |= 5;"
    );
    assert_eq!(fixed.len(), source.len());
}

#[test]
fn reads_comparisons_lambdas_and_coalescing_stay_as_they_are() {
    let source =
        "x = a?.B; if (a?.B == 1) { } f(a?.B => 1); y = a ?? b; z = c ? .5 : 1; w = a?[0];";
    assert!(matches!(prepare(source), std::borrow::Cow::Borrowed(_)));
}

#[test]
fn multi_byte_text_keeps_its_offsets() {
    let source = "// Größe\nlet s = a?.B = 1;";
    assert_eq!(prepare(source).len(), source.len());
}
