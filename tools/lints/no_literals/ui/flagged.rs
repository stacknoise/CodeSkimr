//! Every kind of literal is reported.
#![allow(unused)]

fn numbers() -> u32 {
    let buffer = 4096;
    let ratio = 0.5;
    let zero_float = 0.0;
    let hex = 0x1;
    buffer + hex
}

fn texts() {
    let name = "codeskimr";
    let letter = 'x';
    let raw = b"abc";
    let byte = b'a';
}

fn array() -> [u8; 4] {
    [0; 4]
}

fn patterns(value: u8) -> bool {
    match value {
        2 => true,
        _ => false,
    }
}

fn formats(count: usize) -> String {
    println!("scanned");
    panic_free(format!("{count} files"))
}

fn panic_free(text: String) -> String {
    text
}

macro_rules! local {
    () => {
        7
    };
}

fn local_macro() -> u32 {
    local!()
}

fn main() {}
