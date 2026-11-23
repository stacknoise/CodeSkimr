//! The numbers of the statistics screen, counted when a snapshot is written (ADR 0091).
//!
//! `rebuild` counts the usages and measured values of every type and callable of a snapshot with
//! the calculation of [`load_facts`] and stores one row each in `symbol_statistics`. The rankings
//! then read their first rows through SQL, no matter how large the project is. A snapshot from
//! before the table existed, or counted for other test folders, is counted at its first question.

use std::collections::BTreeMap;
use std::sync::PoisonError;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeStatistics, RankedRows, RankingRequest, StatisticsSetup, StatisticsSummary, UnusedCounts,
};
use codeskimr_domain::{
    Assessment, KindGroup, Ranking, RankingQuery, SnapshotId, SymbolFacts, UsageOrder, Visibility,
    complexity_bars, mark_tests,
};
use rusqlite::{Connection, params};

use crate::codec::column_int;
use crate::consts::{COUNT_ATTEMPTS, NAME_SEPARATOR, PRAGMA_DATA_VERSION};
use crate::error::StorageResult;
use crate::ranking_rows::{Filter, facts_of, filter_of};
use crate::sql;
use crate::statistics::load_facts;
use crate::store::{Build, SqliteStore};

impl SqliteStore {
    /// Makes sure the numbers of `snapshot` exist and were counted for `names`: counts them if
    /// not, once even if several callers ask at the same time.
    fn ensure_statistics(&self, snapshot: SnapshotId, names: &[String]) -> StorageResult<()> {
        let raw = column_int(snapshot.get())?;
        let key = test_key(names);
        if self.read(|conn| counted_for(conn, raw, &key))? {
            return Ok(());
        }
        let gate = self.build_gate(raw, Build::Facts);
        let _counting = gate.lock().unwrap_or_else(PoisonError::into_inner);
        if self.read(|conn| counted_for(conn, raw, &key))? {
            return Ok(());
        }
        self.count_and_store(raw, names)
    }

    /// Counts the numbers on a connection of their own, so the other screens are not held up by
    /// a count that takes long, and stores them in a short write. A write of this store or of
    /// another connection in between (a scan, level 2) may have changed the references that were
    /// counted: then they are counted again, and in the last attempt under the write lock.
    fn count_and_store(&self, snapshot: i64, names: &[String]) -> StorageResult<()> {
        for _ in 0..COUNT_ATTEMPTS {
            let (written, version) = (self.write_count(), self.data_version()?);
            let facts = self.read_apart(|conn| count(conn, snapshot, names))?;
            let stored = self.write(|tx| {
                let unchanged = self.unwritten_since(written) && tx_version(tx)? == version;
                if unchanged {
                    store(tx, snapshot, names, &facts)?;
                }
                Ok(unchanged)
            })?;
            if stored {
                return Ok(());
            }
        }
        self.write(|tx| {
            let facts = count(tx, snapshot, names)?;
            store(tx, snapshot, names, &facts)
        })
    }
}

/// The `data_version` as the connection of a write sees it.
fn tx_version(conn: &Connection) -> StorageResult<i64> {
    Ok(conn.pragma_query_value(None, PRAGMA_DATA_VERSION, |row| row.get(0))?)
}

/// The names of the test folders as one text, to compare what the numbers were counted for.
fn test_key(names: &[String]) -> String {
    names.join(&NAME_SEPARATOR.to_string())
}

/// Whether the numbers of the snapshot exist and were counted for `key`.
fn counted_for(conn: &Connection, snapshot: i64, key: &str) -> StorageResult<bool> {
    let mut statement = conn.prepare_cached(sql::STATISTICS_STATE)?;
    let mut rows = statement.query(params![snapshot])?;
    Ok(match rows.next()? {
        Some(row) => row.get::<_, String>(0)? == key,
        None => false,
    })
}

/// Forgets the numbers of a snapshot, because its references changed; the next question counts
/// them again.
pub(crate) fn forget(conn: &Connection, snapshot: i64) -> StorageResult<()> {
    for delete in sql::STATISTICS_DELETE {
        conn.prepare_cached(delete)?.execute(params![snapshot])?;
    }
    Ok(())
}

/// The symbols of a snapshot with their usage counts and measured values, test code marked for
/// the test folders `names`.
fn count(conn: &Connection, snapshot: i64, names: &[String]) -> StorageResult<Vec<SymbolFacts>> {
    let mut facts = load_facts(conn, snapshot)?;
    mark_tests(&mut facts, names);
    Ok(facts)
}

/// Stores the numbers of a snapshot, instead of the ones it had, as counted for `names`.
fn store(
    conn: &Connection,
    snapshot: i64,
    names: &[String],
    facts: &[SymbolFacts],
) -> StorageResult<()> {
    for delete in sql::STATISTICS_DELETE {
        conn.prepare_cached(delete)?.execute(params![snapshot])?;
    }
    let places = places(facts);
    let mut insert = conn.prepare_cached(sql::STATISTICS_ROW_INSERT)?;
    let mut groups = Groups::default();
    for (fact, place) in facts.iter().zip(places) {
        if KindGroup::All.holds(fact.kind) {
            insert_row(&mut insert, snapshot, place, fact)?;
            groups.count(fact);
        }
    }
    groups.store(conn, snapshot)?;
    let sum = |part: fn(&SymbolFacts) -> u32| facts.iter().map(|fact| i64::from(part(fact))).sum();
    let (certain, probable): (i64, i64) = (
        sum(|fact| fact.usage.certain),
        sum(|fact| fact.usage.probable()),
    );
    conn.execute(
        sql::STATISTICS_STATE_INSERT,
        params![snapshot, test_key(names), certain, probable],
    )?;
    Ok(())
}

/// What a group of the counts of the facets is told apart by: whether the symbol is a type, its
/// language and visibility, whether it is test code, whether it is used, whether no usage means
/// something for it, and whether it has a complexity, a size and a coupling.
type FacetKey = (bool, String, Option<&'static str>, bool, [bool; 5]);

/// The counts per group that the questions of the overview read instead of counting rows.
#[derive(Default)]
struct Groups {
    facets: BTreeMap<FacetKey, i64>,
    /// The callables with a complexity per test code and complexity (its bits, to be a key).
    complexity: BTreeMap<(bool, u64), i64>,
}

impl Groups {
    fn count(&mut self, fact: &SymbolFacts) {
        let measures = &fact.measures;
        let flags = [
            fact.usage.total > 0,
            fact.usage.total == 0 && !fact.is_indirect(),
            measures.complexity.is_some(),
            measures.loc.is_some(),
            measures.efferent.is_some(),
        ];
        let key = (
            KindGroup::Types.holds(fact.kind),
            fact.language.to_string(),
            fact.visibility.map(Visibility::label),
            fact.test,
            flags,
        );
        *self.facets.entry(key).or_default() += 1;
        if let (false, Some(complexity)) = (KindGroup::Types.holds(fact.kind), measures.complexity)
        {
            *self
                .complexity
                .entry((fact.test, complexity.to_bits()))
                .or_default() += 1;
        }
    }

    fn store(&self, conn: &Connection, snapshot: i64) -> StorageResult<()> {
        let mut facet = conn.prepare_cached(sql::STATISTICS_FACET_INSERT)?;
        for ((is_type, language, visibility, test, flags), many) in &self.facets {
            let [used, idle, complexity, loc, efferent] = *flags;
            facet.execute(params![
                snapshot, is_type, language, visibility, test, used, idle, complexity, loc,
                efferent, many
            ])?;
        }
        let mut bars = conn.prepare_cached(sql::STATISTICS_COMPLEXITY_INSERT)?;
        for ((test, bits), many) in &self.complexity {
            bars.execute(params![snapshot, test, f64::from_bits(*bits), many])?;
        }
        Ok(())
    }
}

/// The position of each symbol in the order of path, qualified name and id.
fn places(facts: &[SymbolFacts]) -> Vec<i64> {
    let mut order: Vec<(usize, &SymbolFacts)> = facts.iter().enumerate().collect();
    order.sort_unstable_by(|(_, left), (_, right)| {
        (&left.path, &left.qualified_name, left.id).cmp(&(
            &right.path,
            &right.qualified_name,
            right.id,
        ))
    });
    let mut placed: Vec<(usize, i64)> = order
        .into_iter()
        .zip(0_i64..)
        .map(|((index, _), place)| (index, place))
        .collect();
    placed.sort_unstable();
    placed.into_iter().map(|(_, place)| place).collect()
}

fn insert_row(
    insert: &mut rusqlite::CachedStatement<'_>,
    snapshot: i64,
    place: i64,
    fact: &SymbolFacts,
) -> StorageResult<()> {
    let indirect = fact.is_indirect();
    insert.execute(params![
        snapshot,
        column_int(fact.id.get())?,
        place,
        fact.kind.label(),
        KindGroup::Types.holds(fact.kind),
        fact.language.as_str(),
        fact.visibility.map(Visibility::label),
        fact.test,
        fact.overrides,
        indirect,
        fact.usage.certain,
        fact.usage.total,
        fact.measures.complexity,
        fact.measures.loc,
        fact.measures.maintainability,
        fact.measures.efferent,
        fact.measures.afferent,
    ])?;
    Ok(())
}

impl CodeStatistics for SqliteStore {
    fn symbol_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFacts>, CodedError> {
        let raw = column_int(snapshot.get())?;
        // The load takes long for a large project: it runs on a connection of its own, so the
        // other screens are not held up by it.
        Ok(self.read_apart(|conn| load_facts(conn, raw))?)
    }

    fn prepare(&self, snapshot: SnapshotId, setup: StatisticsSetup<'_>) -> Result<(), CodedError> {
        Ok(self.ensure_statistics(snapshot, setup.test_names)?)
    }

    fn ranked(&self, request: &RankingRequest<'_>) -> Result<RankedRows, CodedError> {
        self.ensure_statistics(request.snapshot, request.setup.test_names)?;
        Ok(self.read(|conn| ranked(conn, request))?)
    }

    fn unused_counts(
        &self,
        snapshot: SnapshotId,
        query: &RankingQuery,
        setup: StatisticsSetup<'_>,
    ) -> Result<UnusedCounts, CodedError> {
        self.ensure_statistics(snapshot, setup.test_names)?;
        Ok(self.read(|conn| unused_counts(conn, snapshot, query, setup))?)
    }

    fn summary(
        &self,
        snapshot: SnapshotId,
        include_tests: bool,
        setup: StatisticsSetup<'_>,
    ) -> Result<StatisticsSummary, CodedError> {
        self.ensure_statistics(snapshot, setup.test_names)?;
        Ok(self.read(|conn| summary(conn, snapshot, include_tests))?)
    }
}

/// The statements of a ranking that is not the one of unused symbols: the ids of its first rows,
/// and how many rows it holds.
fn statements(query: &RankingQuery) -> (&'static str, &'static str) {
    match (query.ranking, query.order) {
        (Ranking::MostUsed, UsageOrder::Certain) => {
            (sql::RANKED_MOST_USED_CERTAIN, sql::COUNT_MOST_USED)
        }
        (Ranking::MostUsed, UsageOrder::All) => (sql::RANKED_MOST_USED_ALL, sql::COUNT_MOST_USED),
        (Ranking::Complexity, _) => (sql::RANKED_COMPLEXITY, sql::COUNT_COMPLEXITY),
        (Ranking::Size, _) => (sql::RANKED_SIZE, sql::COUNT_SIZE),
        (Ranking::Coupling | Ranking::Unused, _) => (sql::RANKED_COUPLING, sql::COUNT_COUPLING),
    }
}

/// The assessments in the order the ranking of unused symbols lists them.
const ASSESSMENT_ORDER: [Assessment; 3] = [
    Assessment::Likely,
    Assessment::Public,
    Assessment::Approximate,
];

fn ranked(conn: &Connection, request: &RankingRequest<'_>) -> StorageResult<RankedRows> {
    let mut filter = filter_of(request);
    let (ids, total) = if request.query.ranking == Ranking::Unused {
        (
            unused_ids(conn, request, &mut filter)?,
            unused_total(conn, request, &filter)?,
        )
    } else {
        let (ids_sql, count_sql) = statements(request.query);
        (filter.ids(conn, ids_sql)?, filter.count(conn, count_sql)?)
    };
    Ok(RankedRows {
        rows: facts_of(conn, request.snapshot, &ids)?,
        total,
    })
}

/// The ids of the first rows of the unused symbols: those of each assessment in the order of
/// their place, the very likely unused ones first, only the wanted assessment if one is asked.
fn unused_ids(
    conn: &Connection,
    request: &RankingRequest<'_>,
    filter: &mut Filter,
) -> StorageResult<Vec<i64>> {
    let mut ids = Vec::new();
    for assessment in ASSESSMENT_ORDER {
        if request
            .assessment
            .is_some_and(|wanted| wanted != assessment)
        {
            continue;
        }
        let missing = request.limit.saturating_sub(ids.len());
        if missing == 0 {
            break;
        }
        filter.select(assessment, missing);
        ids.extend(filter.ids(conn, sql::RANKED_UNUSED)?);
    }
    Ok(ids)
}

/// How many unused symbols the ranking holds, from the counts per group of visibility and
/// language, judged as the screen judges them.
fn unused_total(
    conn: &Connection,
    request: &RankingRequest<'_>,
    filter: &Filter,
) -> StorageResult<usize> {
    let mut total = 0_usize;
    for (visibility, language, test, many) in filter.unused_groups(conn)? {
        let approximate = request.setup.approximate.contains(&language);
        let kind = Assessment::of(visibility, approximate);
        let wanted = request.assessment.is_none_or(|wanted| wanted == kind);
        if wanted && (request.query.include_tests || !test) {
            total = total.saturating_add(usize::try_from(many).unwrap_or(usize::MAX));
        }
    }
    Ok(total)
}

fn unused_counts(
    conn: &Connection,
    snapshot: SnapshotId,
    query: &RankingQuery,
    setup: StatisticsSetup<'_>,
) -> StorageResult<UnusedCounts> {
    let filter = filter_of(&RankingRequest {
        snapshot,
        query,
        assessment: None,
        limit: 0,
        setup,
    });
    let mut counts = UnusedCounts::default();
    for (visibility, language, test, many) in filter.unused_groups(conn)? {
        let approximate = setup.approximate.contains(&language);
        let kind = Assessment::of(visibility, approximate);
        let slot = match kind {
            Assessment::Likely => &mut counts.likely,
            Assessment::Public => &mut counts.public,
            Assessment::Approximate => &mut counts.approximate,
        };
        if test && !query.include_tests {
            counts.tests_hidden = counts.tests_hidden.saturating_add(many);
        } else {
            *slot = slot.saturating_add(many);
        }
    }
    Ok(counts)
}

fn summary(
    conn: &Connection,
    snapshot: SnapshotId,
    include_tests: bool,
) -> StorageResult<StatisticsSummary> {
    let raw = column_int(snapshot.get())?;
    let filter = Filter::all(raw, include_tests);
    let mut state = conn.prepare_cached(sql::STATISTICS_STATE)?;
    let (certain, probable): (i64, i64) =
        state.query_row(params![raw], |row| Ok((row.get(1)?, row.get(2)?)))?;
    Ok(StatisticsSummary {
        ranked: u32::try_from(filter.count(conn, sql::COUNT_RANKED)?).unwrap_or(u32::MAX),
        certain_references: u64::try_from(certain).unwrap_or_default(),
        probable_references: u64::try_from(probable).unwrap_or_default(),
        histogram: complexity_bars(filter.complexity(conn)?),
    })
}
