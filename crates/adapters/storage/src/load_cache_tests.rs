//! The larger page cache of the write of a large scan: raised for a large load, the usual size
//! again afterwards, and left alone for a small load.

use rusqlite::Connection;

use crate::consts::{
    CACHE_SIZE_KIB, LARGE_LOAD_REFERENCES_MIN, LOAD_CACHE_SIZE_KIB, PRAGMA_CACHE_SIZE,
};
use crate::snapshots::LoadCache;

const SMALL_LOAD: usize = 10;

fn cache_size(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, PRAGMA_CACHE_SIZE, |row| row.get(0))
        .unwrap()
}

#[test]
fn a_large_load_raises_the_cache_and_the_usual_size_comes_back() {
    let conn = Connection::open_in_memory().unwrap();
    {
        let _cache = LoadCache::raise(&conn, LARGE_LOAD_REFERENCES_MIN).unwrap();
        assert_eq!(cache_size(&conn), LOAD_CACHE_SIZE_KIB);
    }
    assert_eq!(cache_size(&conn), CACHE_SIZE_KIB);
}

#[test]
fn a_small_load_keeps_the_cache() {
    let conn = Connection::open_in_memory().unwrap();
    let before = cache_size(&conn);
    {
        let _cache = LoadCache::raise(&conn, SMALL_LOAD).unwrap();
        assert_eq!(cache_size(&conn), before);
    }
    assert_eq!(cache_size(&conn), before);
}
