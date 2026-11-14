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
fn main() {}
