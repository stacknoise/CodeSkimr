//! The decoder: valid input, unknown fields, and every way an index can be broken.

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_scip::{
    Corruption, PositionEncoding, ScipError, decode_index, normalize_path, read_index,
};

use crate::common::{
    LIMIT, document, index, len_field, nested_groups, occurrence, sample_index, scip, varint,
    varint_field,
};

fn corruption(result: Result<codeskimr_scip::ScipIndex, ScipError>) -> Corruption {
    match result {
        Err(ScipError::Corrupt(detail)) => detail,
        other => panic!("expected a corrupt index, got {other:?}"),
    }
}

#[test]
fn a_valid_index_is_decoded() {
    let decoded = decode_index(&sample_index(), LIMIT).expect("decodes");
    assert_eq!(decoded.project_root, "file:///work/project");
    let [document] = decoded.documents.as_slice() else {
        panic!("one document");
    };
    assert_eq!(document.path.as_str(), "src/a.cs");
    assert_eq!(document.encoding, PositionEncoding::Unspecified);
    let [def, call] = document.occurrences.as_slice() else {
        panic!("two occurrences");
    };
    assert!(def.is_definition());
    assert!(!call.is_definition());
    assert_eq!(def.symbol, call.symbol);
    assert_eq!(
        decoded.symbol(def.symbol),
        Some(scip("ns/Svc#Run().").as_str())
    );
    assert_eq!((def.range.start().line, def.range.end().column), (2, 19));
    assert_eq!((call.range.start().line, call.range.end().line), (7, 9));
}

#[test]
fn the_position_encoding_is_kept() {
    let mut doc = document("a.rs", &[]);
    doc.extend(varint_field(6, 1));
    let decoded = decode_index(&index("", &[doc]), LIMIT).expect("decodes");
    let encodings: Vec<_> = decoded.documents.iter().map(|d| d.encoding).collect();
    assert_eq!(encodings, [PositionEncoding::Utf8]);
}

#[test]
fn an_unpacked_range_is_read_too() {
    let mut occ = Vec::new();
    for number in [4_u64, 8, 12] {
        occ.extend(varint_field(1, number));
    }
    occ.extend(len_field(2, b"local 1"));
    let doc = document("a.rs", &[occ]);
    let decoded = decode_index(&index("", &[doc]), LIMIT).expect("decodes");
    let range = decoded
        .documents
        .first()
        .and_then(|d| d.occurrences.first());
    assert_eq!(range.map(|o| o.range.end().column), Some(12));
}

#[test]
fn unknown_fields_of_every_wire_type_are_skipped() {
    let mut occ = occurrence(&[1, 2, 3], "local 1", 0);
    occ.extend(varint_field(40, 7));
    occ.extend(len_field(41, b"ignored"));
    occ.extend([0xd5, 0x02, 1, 2, 3, 4]);
    occ.extend([0xd1, 0x02, 1, 2, 3, 4, 5, 6, 7, 8]);
    occ.extend(nested_groups(42, 3));
    let mut doc = document("a.rs", &[occ]);
    doc.extend(len_field(50, b"language data"));
    let mut bytes = index("root", &[doc]);
    bytes.extend(len_field(3, b"external symbols"));
    let decoded = decode_index(&bytes, LIMIT).expect("decodes");
    assert_eq!(decoded.documents.len(), 1);
}

#[test]
fn a_truncated_index_is_corrupt() {
    let bytes = sample_index();
    let cut = bytes.get(..bytes.len() - 1).expect("shorter");
    assert_eq!(corruption(decode_index(cut, LIMIT)), Corruption::Truncated);
    for end in 0..bytes.len() {
        let result = decode_index(bytes.get(..end).expect("prefix"), LIMIT);
        assert!(result.is_ok() || matches!(result, Err(ScipError::Corrupt(_))));
    }
}

#[test]
fn an_index_above_the_limit_is_refused_before_it_is_read() {
    let bytes = sample_index();
    let limit = bytes.len() as u64 - 1;
    let error = decode_index(&bytes, limit).expect_err("too large");
    assert!(matches!(error, ScipError::TooLarge { .. }));
    assert_eq!(error.code(), ErrorCode::ScipIndexTooLarge);
    assert!(decode_index(&bytes, limit + 1).is_ok());
}

#[test]
fn a_file_is_read_within_its_limit() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("index.scip");
    std::fs::write(&path, sample_index()).expect("write");
    let size = sample_index().len() as u64;
    assert!(read_index(&path, size).is_ok());
    let error = read_index(&path, size - 1).expect_err("too large");
    assert_eq!(error.code(), ErrorCode::ScipIndexTooLarge);
}

#[test]
fn a_missing_file_has_the_invalid_index_code() {
    let dir = tempfile::tempdir().expect("temp dir");
    let error = read_index(&dir.path().join("none.scip"), LIMIT).expect_err("missing");
    assert!(matches!(error, ScipError::Unreadable(_)));
    assert_eq!(error.code(), ErrorCode::ScipIndexInvalid);
}

#[test]
fn unsafe_paths_are_refused() {
    for raw in [
        "../etc/passwd",
        "a/../../b",
        "a/..",
        "/etc/passwd",
        "\\\\server\\share\\a.cs",
        "C:\\work\\a.cs",
        "c:/work/a.cs",
        "",
        ".",
        "a\0b",
    ] {
        assert!(normalize_path(raw).is_err(), "{raw:?} must be refused");
        let doc = document(raw, &[]);
        let result = decode_index(&index("", &[doc]), LIMIT);
        assert!(matches!(result, Err(ScipError::Corrupt(_))), "{raw:?}");
    }
}

#[test]
fn clean_paths_are_normalised() {
    for (raw, clean) in [
        ("src/a.cs", "src/a.cs"),
        ("./src//a.cs", "src/a.cs"),
        ("src\\sub\\a.cs", "src/sub/a.cs"),
        ("a/./b/", "a/b"),
        ("dir/file..cs", "dir/file..cs"),
    ] {
        assert_eq!(normalize_path(raw).expect("clean").as_str(), clean);
    }
}

#[test]
fn deeply_nested_groups_are_refused() {
    let mut doc = document("a.rs", &[]);
    doc.extend(nested_groups(60, 200));
    let result = decode_index(&index("", &[doc]), LIMIT);
    assert_eq!(corruption(result), Corruption::TooDeep);
}

#[test]
fn an_end_group_without_a_start_is_corrupt() {
    let mut bytes = sample_index();
    bytes.extend(varint((60 << 3) | 4));
    assert_eq!(
        corruption(decode_index(&bytes, LIMIT)),
        Corruption::WireType
    );
}

#[test]
fn broken_ranges_are_corrupt() {
    let symbol = "local 1";
    let negative = [0xff_u8; 9].into_iter().chain([1]).collect::<Vec<_>>();
    let mut negative_range = len_field(1, &negative);
    negative_range.extend(len_field(2, symbol.as_bytes()));
    let cases = [
        (occurrence(&[1, 2], symbol, 0), Corruption::BadRange),
        (
            occurrence(&[1, 2, 3, 4, 5], symbol, 0),
            Corruption::BadRange,
        ),
        (occurrence(&[5, 9, 5, 2], symbol, 0), Corruption::BadRange),
        (occurrence(&[6, 0, 5, 9], symbol, 0), Corruption::BadRange),
        (negative_range, Corruption::NumberOutOfRange),
    ];
    for (occ, expected) in cases {
        let doc = document("a.rs", &[occ]);
        assert_eq!(
            corruption(decode_index(&index("", &[doc]), LIMIT)),
            expected
        );
    }
}

#[test]
fn bad_text_and_wrong_wire_types_are_corrupt() {
    let bad_utf8 = len_field(2, &[0xff, 0xfe]);
    let mut occ = len_field(1, &[1, 2, 3]);
    occ.extend(bad_utf8);
    let doc = len_field(2, &len_field(2, &occ));
    assert_eq!(corruption(decode_index(&doc, LIMIT)), Corruption::NotUtf8);

    let long = "s".repeat(70_000);
    let doc = document("a.rs", &[occurrence(&[1, 2, 3], &long, 0)]);
    assert_eq!(
        corruption(decode_index(&index("", &[doc]), 1 << 22)),
        Corruption::TextTooLong
    );

    let wrong = varint_field(2, 5);
    assert_eq!(
        corruption(decode_index(&wrong, LIMIT)),
        Corruption::WireType
    );
}

#[test]
fn a_document_without_a_path_is_corrupt() {
    let doc = len_field(2, &len_field(2, &occurrence(&[1, 2, 3], "local 1", 0)));
    assert!(matches!(
        decode_index(&doc, LIMIT),
        Err(ScipError::Corrupt(Corruption::UnsafePath))
    ));
}

#[test]
fn bits_beyond_sixty_four_in_the_tenth_varint_byte_are_refused() {
    let mut bytes = vec![0xff_u8; 9];
    bytes.push(0x02);
    assert_eq!(
        corruption(decode_index(&bytes, LIMIT)),
        Corruption::VarintTooLong
    );
    let mut longest = vec![0xff_u8; 9];
    longest.push(0x01);
    assert!(!matches!(
        decode_index(&longest, LIMIT),
        Err(ScipError::Corrupt(Corruption::VarintTooLong))
    ));
}

#[test]
fn an_overlong_varint_is_corrupt() {
    let bytes = vec![0x80_u8; 12];
    assert_eq!(
        corruption(decode_index(&bytes, LIMIT)),
        Corruption::VarintTooLong
    );
}

#[cfg(unix)]
#[test]
fn a_link_in_place_of_the_index_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("real.scip");
    std::fs::write(&target, b"").unwrap();
    let link = dir.path().join("index.scip");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let error = codeskimr_scip::read_index(&link, 1 << 20).unwrap_err();
    assert!(matches!(error, codeskimr_scip::ScipError::Unreadable(_)));
    assert!(codeskimr_scip::read_index(&target, 1 << 20).is_ok());
}
