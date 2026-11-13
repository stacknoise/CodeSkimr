//! Nothing in here is reported.
#![allow(unused)]

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i64,
    y: i64,
}

fn free_values() -> i64 {
    let zero = 0;
    let one = 1u8;
    let negative = -1;
    let flag = true;
    zero + negative + i64::from(one)
}

fn plain_format(value: u32) -> String {
    format!("{}", value)
}

fn assertions(value: u32) {
    assert!(value < u32::MAX);
}

#[cfg(test)]
mod tests {
    #[test]
    fn literals_in_tests() {
        assert_eq!(2 + 2, 4);
        let _ = "text";
    }
}

#[test]
fn single_test() {
    let _ = 'c';
}

/// Doc comments are attributes, not literals.
#[deprecated(note = "attribute arguments are not value literals")]
#[path = "exempt_path.rs"]
mod documented {}

macro_rules! documented_struct {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        struct $name;
    };
}

documented_struct!(
    /// A struct whose doc comment travels through a macro.
    Documented
);

fn main() {}
