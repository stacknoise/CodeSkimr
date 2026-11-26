//! The pure parts of the rename preview: names, whole-word matching and the kind of a text line.

use codeskimr_application::rename::{TextKind, classify, find_word, is_valid_name};

#[test]
fn a_new_name_is_letters_digits_and_underscores_not_starting_with_a_digit() {
    for good in ["Submit", "_x", "a1", "Größe", "snake_case"] {
        assert!(is_valid_name(good), "{good}");
    }
    for bad in ["", " ", "1a", "a b", "a-b", "a.b", "a()"] {
        assert!(!is_valid_name(bad), "{bad:?}");
    }
}

#[test]
fn a_word_is_found_only_as_a_whole() {
    assert_eq!(find_word("x.PlaceOrder(y)", "PlaceOrder"), Some(2));
    assert_eq!(
        find_word("PlaceOrderAsync(); _PlaceOrder; PlaceOrder", "PlaceOrder"),
        Some(32)
    );
    assert_eq!(find_word("PlaceOrder2 xPlaceOrder", "PlaceOrder"), None);
    assert_eq!(find_word("anything", ""), None);
    assert_eq!(find_word("ÄPlaceOrder", "PlaceOrder"), None);
}

#[test]
fn a_line_is_told_by_its_file_and_by_how_it_is_written() {
    let name = "PlaceOrder";
    let kind = |path: &str, line: &str| classify(path, line, name);
    assert_eq!(
        kind("docs/api.md", "POST /orders calls PlaceOrder"),
        TextKind::Documentation
    );
    assert_eq!(
        kind("Api/appsettings.JSON", "\"audit\": [\"PlaceOrder\"]"),
        TextKind::Configuration
    );
    assert_eq!(kind("a.cs", "nameof(PlaceOrder)"), TextKind::Nameof);
    assert_eq!(
        kind("a.cs", "    // PlaceOrder is called twice"),
        TextKind::Comment
    );
    assert_eq!(kind("a.py", "# PlaceOrder"), TextKind::Comment);
    assert_eq!(kind("a.cs", "Log(\"PlaceOrder failed\");"), TextKind::Text);
    assert_eq!(
        kind("a.cs", "var m = \"x\"; PlaceOrder(m);"),
        TextKind::Unresolved
    );
    assert_eq!(
        kind("a.cs", "Task<OrderId> PlaceOrder(Order order);"),
        TextKind::Unresolved
    );
    assert_eq!(kind("Makefile", "PlaceOrder:"), TextKind::Unresolved);
}
