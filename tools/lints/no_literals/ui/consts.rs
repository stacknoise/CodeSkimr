//! In a `consts.rs` module only the value side of `const` and `static` may hold literals.
#![allow(unused)]

const BUFFER_SIZE: usize = 4096;
static PROTOCOL: &str = "stdio";
const LIMITS: [u32; 2] = [10, 20];

fn outside() -> usize {
    BUFFER_SIZE + 8
}

fn main() {}
