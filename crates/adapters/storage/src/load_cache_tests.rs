//! The larger page cache of the write of a large scan: raised for a large load, the size it had
//! again afterwards (also after a rollback or a failure), and left alone for a small load.

use rusqlite::Connection;

use crate::consts::{
    LARGE_LOAD_REFERENCES_MIN, LOAD_CACHE_SIZE_KIB, PRAGMA_CACHE_SIZE, SORT_CACHE_SIZE_KIB,
};
use crate::error::StorageResult;
use crate::snapshots::{LoadCache, with_sort_cache};

const SMALL_LOAD: usize = 10;
/// A size that is neither the load size nor the default, so that a restore to a constant shows.
const USUAL_KIB: i64 = -12_345;

fn connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, PRAGMA_CACHE_SIZE, USUAL_KIB)
        .unwrap();
    conn
}

fn cache_size(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, PRAGMA_CACHE_SIZE, |row| row.get(0))
        .unwrap()
}

#[test]
fn a_large_load_raises_the_cache_and_the_size_it_had_comes_back() {
    let conn = connection();
    {
        let _cache = LoadCache::raise(&conn, LARGE_LOAD_REFERENCES_MIN).unwrap();
        assert_eq!(cache_size(&conn), LOAD_CACHE_SIZE_KIB);
    }
    assert_eq!(cache_size(&conn), USUAL_KIB);
}

#[test]
fn a_small_load_keeps_the_cache() {
    let conn = connection();
    {
        let _cache = LoadCache::raise(&conn, SMALL_LOAD).unwrap();
        assert_eq!(cache_size(&conn), USUAL_KIB);
    }
    assert_eq!(cache_size(&conn), USUAL_KIB);
}

#[test]
fn a_rolled_back_transaction_does_not_leave_the_cache_raised() {
    let mut conn = connection();
    let tx = conn.transaction().unwrap();
    {
        let _cache = LoadCache::raise(&tx, LARGE_LOAD_REFERENCES_MIN).unwrap();
        assert_eq!(cache_size(&tx), LOAD_CACHE_SIZE_KIB);
    }
    tx.rollback().unwrap();
    assert_eq!(cache_size(&conn), USUAL_KIB);
}

#[test]
fn a_write_that_fails_does_not_leave_the_cache_raised() {
    let conn = connection();
    let result: StorageResult<()> = (|| {
        let _cache = LoadCache::raise(&conn, LARGE_LOAD_REFERENCES_MIN)?;
        conn.execute("SELECT * FROM a_table_that_is_not_there", [])?;
        Ok(())
    })();
    assert!(result.is_err());
    assert_eq!(cache_size(&conn), USUAL_KIB);
}

#[test]
fn building_an_index_uses_the_small_cache_and_gives_the_size_back() {
    let conn = connection();
    let inside = with_sort_cache(&conn, || Ok(cache_size(&conn))).unwrap();
    assert_eq!(inside, SORT_CACHE_SIZE_KIB);
    assert_eq!(cache_size(&conn), USUAL_KIB);
    let failed: Result<(), _> =
        with_sort_cache(&conn, || Err(crate::error::StorageError::ProjectNotFound));
    assert!(failed.is_err());
    assert_eq!(cache_size(&conn), USUAL_KIB);
}
