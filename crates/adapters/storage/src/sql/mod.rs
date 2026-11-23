//! SQL statements, defined in `consts.rs` next to this file.

mod consts;

pub use consts::*;

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use crate::store::SqliteStore;

    /// The statements that read the references of one snapshot.
    const OF_ONE_SNAPSHOT: [&str; 6] = [
        super::REFERENCE_TALLY_ROWS,
        super::REFERENCE_ENDS_CERTAIN,
        super::REFERENCE_TARGETS,
        super::REFERENCES_OF_SNAPSHOT,
        super::VERIFIED_REFERENCE_FILES,
        super::REFERENCE_PAIRS,
    ];

    fn plan(store: &SqliteStore, statement: &str) -> String {
        store
            .read(|conn| {
                let mut explained = conn.prepare(&format!("EXPLAIN QUERY PLAN {statement}"))?;
                let filled = vec![rusqlite::types::Null; explained.parameter_count()];
                let steps = explained
                    .query_map(rusqlite::params_from_iter(filled), |row| {
                        row.get::<_, String>(3)
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(steps.join("\n"))
            })
            .unwrap()
    }

    /// The references of every project share one table. A statement that walks the whole table
    /// makes a small project wait for the references of the largest one in the database.
    #[test]
    fn the_references_of_a_snapshot_are_read_through_its_files_not_the_whole_table() {
        let store = SqliteStore::open_in_memory(NonZeroU32::new(3).unwrap()).unwrap();
        for statement in OF_ONE_SNAPSHOT {
            let plan = plan(&store, statement);
            assert!(
                !plan.contains("SCAN symbol_references"),
                "{statement}\n{plan}"
            );
        }
    }
}
