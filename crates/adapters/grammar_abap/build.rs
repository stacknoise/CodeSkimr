//! Compiles the vendored C source of the grammar.

fn main() {
    let dir = std::path::Path::new("vendor");
    let mut build = cc::Build::new();
    build
        .include(dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .warnings(false);
    build.compile("tree_sitter_abap");
    println!("cargo:rerun-if-changed=vendor");
}
