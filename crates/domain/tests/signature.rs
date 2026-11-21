//! The parameters of a signature as written.

use codeskimr_domain::signature_parameters;

#[test]
fn the_parameters_are_split_at_the_commas_outside_brackets() {
    assert_eq!(
        signature_parameters("int Total(int a, Map<string, int> b, (int, int) c)"),
        ["int a", "Map<string, int> b", "(int, int) c"]
    );
}

#[test]
fn an_arrow_in_a_parameter_closes_no_bracket() {
    assert_eq!(
        signature_parameters("fn run(f: Box<dyn Fn(i32) -> i32>, n: u8) -> bool"),
        ["f: Box<dyn Fn(i32) -> i32>", "n: u8"]
    );
}

#[test]
fn only_the_first_list_counts_and_none_gives_nothing() {
    assert_eq!(signature_parameters("void Open();"), Vec::<String>::new());
    assert_eq!(signature_parameters("int Size"), Vec::<String>::new());
    assert_eq!(signature_parameters("f(a)(b)"), ["a"]);
}
