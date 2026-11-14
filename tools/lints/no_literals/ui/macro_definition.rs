//! A `macro_rules!` is checked where it is defined, also when this crate never uses it.
#![allow(unused)]

#[macro_export]
macro_rules! limit {
    () => {
        4096
    };
}

macro_rules! nested {
    () => {
        [0, 1, 2]
    };
}

macro_rules! free {
    () => {
        (0, 1, true)
    };
}

macro_rules! passes_through {
    ($value:expr) => {
        $value + 1
    };
}


macro_rules! logs {
    () => {
        println!("scanned")
    };
}

fn uses() {
    let _ = limit!();
    let _ = limit!();
    logs!();
    logs!();
}

// Only the transcriber (right of `=>`) is checked, and only value literals in it.
macro_rules! matcher_side {
    (2) => {
        0
    };
    ("on") => {
        1
    };
}

macro_rules! syntax_only {
    () => {
        #[cfg(feature = "x")]
        extern "C" fn callback() {}
        const _: &str = env!("CARGO_PKG_NAME");
        compile_error!("not supported");
    };
}

macro_rules! joined {
    () => {
        concat!("code", "skimr")
    };
}

macro_rules! outer {
    () => {
        macro_rules! inner {
            () => {
                9
            };
        }
    };
}

macro_rules! makes_tests {
    () => {
        #[test]
        fn generated() {
            assert_eq!(2 + 2, 4);
        }
    };
}

fn more_uses() {
    let _ = joined!();
    let _ = joined!();
    outer!();
    makes_tests!();
}

fn main() {}
