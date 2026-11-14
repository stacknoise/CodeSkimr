//! Hand-built Protobuf bytes of an index and the model types of a snapshot.

use codeskimr_domain::{
    Confidence, ContentHash, FileId, LanguageId, Loc, Position, QualifiedName, Reference,
    ReferenceId, ReferenceKind, RelativePath, SnapshotId, SourceFile, Span, Symbol, SymbolId,
    SymbolKind, SymbolName,
};

/// A limit that no test index reaches.
pub const LIMIT: u64 = 1 << 20;

/// Tag of a field with the wire type.
fn tag(field: u32, wire: u64) -> Vec<u8> {
    varint((u64::from(field) << 3) | wire)
}

pub fn varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let low = u8::try_from(value & 0x7f).expect("seven bits");
        value >>= 7;
        if value == 0 {
            bytes.push(low);
            return bytes;
        }
        bytes.push(low | 0x80);
    }
}

pub fn len_field(field: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag(field, 2);
    bytes.extend(varint(payload.len() as u64));
    bytes.extend(payload);
    bytes
}

pub fn varint_field(field: u32, value: u64) -> Vec<u8> {
    let mut bytes = tag(field, 0);
    bytes.extend(varint(value));
    bytes
}

/// A group of fields with the given nesting depth, as an unknown field would carry it.
pub fn nested_groups(field: u32, depth: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    for _ in 0..depth {
        bytes.extend(tag(field, 3));
    }
    for _ in 0..depth {
        bytes.extend(tag(field, 4));
    }
    bytes
}

/// An occurrence with a packed range.
pub fn occurrence(range: &[u32], symbol: &str, roles: u32) -> Vec<u8> {
    let packed: Vec<u8> = range.iter().flat_map(|n| varint(u64::from(*n))).collect();
    let mut bytes = len_field(1, &packed);
    bytes.extend(len_field(2, symbol.as_bytes()));
    if roles != 0 {
        bytes.extend(varint_field(3, u64::from(roles)));
    }
    bytes
}

pub fn definition(range: &[u32], symbol: &str) -> Vec<u8> {
    occurrence(range, symbol, 1)
}

pub fn usage(range: &[u32], symbol: &str) -> Vec<u8> {
    occurrence(range, symbol, 0)
}

pub fn document(path: &str, occurrences: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = len_field(1, path.as_bytes());
    for occurrence in occurrences {
        bytes.extend(len_field(2, occurrence));
    }
    bytes
}

pub fn index(root: &str, documents: &[Vec<u8>]) -> Vec<u8> {
    let metadata = len_field(3, root.as_bytes());
    let mut bytes = len_field(1, &metadata);
    for document in documents {
        bytes.extend(len_field(2, document));
    }
    bytes
}

/// The index of a small project: the definition of `Svc.Run` in `src/a.cs` and a usage of it at
/// each of `calls` in `src/b.cs`.
pub fn run_index(calls: &[[u32; 3]]) -> Vec<u8> {
    let symbol = scip("ns/Svc#Run().");
    let usages: Vec<Vec<u8>> = calls.iter().map(|range| usage(range, &symbol)).collect();
    index(
        "file:///p",
        &[
            document("src/a.cs", &[definition(&[2, 16, 19], &symbol)]),
            document("src/b.cs", &usages),
        ],
    )
}

/// A SCIP symbol of the C# indexer's shape around `descriptors`.
pub fn scip(descriptors: &str) -> String {
    format!("scip-dotnet nuget . . {descriptors}")
}

pub fn pos(line: u32, column: u32) -> Position {
    Position { line, column }
}

pub fn span(start: (u32, u32), end: (u32, u32)) -> Span {
    Span::new(pos(start.0, start.1), pos(end.0, end.1)).expect("ordered span")
}

pub fn file(id: u64, path: &str) -> SourceFile {
    SourceFile {
        id: FileId::new(id),
        snapshot: SnapshotId::new(1),
        path: RelativePath::new(path).expect("path"),
        language: LanguageId::new("csharp").expect("language"),
        content_hash: ContentHash::new("hash").expect("hash"),
        loc: Loc(10),
    }
}

pub fn symbol(
    id: u64,
    file: u64,
    kind: SymbolKind,
    name: &str,
    (start, end): ((u32, u32), (u32, u32)),
) -> Symbol {
    Symbol {
        id: SymbolId::new(id),
        file: FileId::new(file),
        kind,
        name: SymbolName::new(name).expect("name"),
        qualified_name: QualifiedName::new(format!("App.{name}")).expect("qualified name"),
        parent: None,
        visibility: None,
        is_static: false,
        span: span(start, end),
        signature: None,
        doc: None,
    }
}

/// A level 1 reference at `at` (line, column) of one column width.
pub fn level_one(
    id: u64,
    (from, to): (u64, u64),
    file: u64,
    at: (u32, u32),
    kind: ReferenceKind,
) -> Reference {
    Reference {
        kind,
        span: span(at, (at.0, at.1 + 1)),
        file: FileId::new(file),
        to: SymbolId::new(to),
        from: SymbolId::new(from),
        id: ReferenceId::new(id),
        ..unchecked_guess()
    }
}

fn unchecked_guess() -> Reference {
    Reference {
        confidence: Confidence::Probable,
        verified: false,
        id: ReferenceId::new(0),
        from: SymbolId::new(0),
        to: SymbolId::new(0),
        file: FileId::new(0),
        span: span((0, 0), (0, 1)),
        kind: ReferenceKind::Read,
    }
}

/// The index that `sample_index` documents: one definition and one usage of `Svc.Run`.
pub fn sample_index() -> Vec<u8> {
    let symbol = scip("ns/Svc#Run().");
    let doc = document(
        "src/a.cs",
        &[
            definition(&[2, 16, 19], &symbol),
            usage(&[7, 4, 9, 6], &symbol),
        ],
    );
    index("file:///work/project", &[doc])
}
