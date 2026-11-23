//! Reading the rankings from `symbol_statistics` (ADR 0091): the ids of the first rows, how many
//! there are, and the facts of the rows that are shown.

use codeskimr_application::ports::RankingRequest;
use codeskimr_domain::{
    Assessment, KindGroup, LanguageId, Measures, QualifiedName, RelativePath, SnapshotId,
    SymbolFacts, SymbolId, SymbolName, UsageTally, Visibility,
};
use rusqlite::{Connection, Statement, ToSql};

use crate::codec::{Cols, column_int, id_from, id_list, label_from, small_from};
use crate::consts::{
    ASSESSMENT_NUMBER_APPROXIMATE, ASSESSMENT_NUMBER_LIKELY, ASSESSMENT_NUMBER_PUBLIC,
    NAME_SEPARATOR, PARAM_APPROXIMATE, PARAM_ASSESSMENT, PARAM_HIDDEN_FIRST, PARAM_HIDDEN_SECOND,
    PARAM_HIDDEN_THIRD, PARAM_IDS, PARAM_IS_TYPE, PARAM_LANGUAGE, PARAM_LIMIT, PARAM_SNAPSHOT,
    PARAM_TESTS,
};
use crate::error::StorageResult;
use crate::sql;

/// What a statement over `symbol_statistics` is asked: the values of its parameters.
pub(crate) struct Filter {
    snapshot: i64,
    is_type: Option<i64>,
    language: Option<String>,
    tests: i64,
    assessment: Option<i64>,
    approximate: String,
    limit: i64,
}

/// The filter of a ranking.
pub(crate) fn filter_of(request: &RankingRequest<'_>) -> Filter {
    let query = request.query;
    let separator = NAME_SEPARATOR.to_string();
    let mut approximate = separator.clone();
    for language in request.setup.approximate {
        approximate.push_str(language);
        approximate.push_str(&separator);
    }
    Filter {
        snapshot: column_int(request.snapshot.get()).unwrap_or(i64::MAX),
        is_type: match query.kinds {
            KindGroup::All => None,
            KindGroup::Types => Some(1),
            KindGroup::Functions => Some(0),
        },
        language: query.language.as_ref().map(ToString::to_string),
        tests: i64::from(query.include_tests),
        assessment: request.assessment.map(number_of),
        approximate,
        limit: i64::try_from(request.limit).unwrap_or(i64::MAX),
    }
}

const fn number_of(assessment: Assessment) -> i64 {
    match assessment {
        Assessment::Likely => ASSESSMENT_NUMBER_LIKELY,
        Assessment::Public => ASSESSMENT_NUMBER_PUBLIC,
        Assessment::Approximate => ASSESSMENT_NUMBER_APPROXIMATE,
    }
}

impl Filter {
    /// Asks for the symbols with `assessment` and at most `limit` of them.
    pub(crate) fn select(&mut self, assessment: Assessment, limit: usize) {
        self.assessment = Some(number_of(assessment));
        self.limit = i64::try_from(limit).unwrap_or(i64::MAX);
    }

    /// The filter of the sums of the overview: every kind and language.
    pub(crate) fn all(snapshot: i64, include_tests: bool) -> Self {
        Self {
            snapshot,
            is_type: None,
            language: None,
            tests: i64::from(include_tests),
            assessment: None,
            approximate: String::new(),
            limit: 0,
        }
    }

    fn bind(&self, statement: &mut rusqlite::Statement<'_>) -> StorageResult<()> {
        let hidden = |visibility: Visibility| visibility.label();
        let values: [(&str, &dyn ToSql); 10] = [
            (PARAM_SNAPSHOT, &self.snapshot),
            (PARAM_IS_TYPE, &self.is_type),
            (PARAM_LANGUAGE, &self.language),
            (PARAM_TESTS, &self.tests),
            (PARAM_ASSESSMENT, &self.assessment),
            (PARAM_APPROXIMATE, &self.approximate),
            (PARAM_LIMIT, &self.limit),
            (PARAM_HIDDEN_FIRST, &hidden(Visibility::Private)),
            (PARAM_HIDDEN_SECOND, &hidden(Visibility::Internal)),
            (PARAM_HIDDEN_THIRD, &hidden(Visibility::Protected)),
        ];
        bind(statement, &values)
    }

    /// The ids the statement returns in its first column, in its order.
    pub(crate) fn ids(&self, conn: &Connection, statement: &str) -> StorageResult<Vec<i64>> {
        let mut prepared = conn.prepare_cached(statement)?;
        self.bind(&mut prepared)?;
        let mut rows = prepared.raw_query();
        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            ids.push(row.get(0)?);
        }
        Ok(ids)
    }

    /// The number the statement counts.
    pub(crate) fn count(&self, conn: &Connection, statement: &str) -> StorageResult<usize> {
        let mut prepared = conn.prepare_cached(statement)?;
        self.bind(&mut prepared)?;
        let mut rows = prepared.raw_query();
        let counted: i64 = rows.next()?.map_or(Ok(0), |row| row.get(0))?;
        Ok(usize::try_from(counted).unwrap_or_default())
    }

    /// The symbols without usages by visibility, language and test code, with how many there are.
    pub(crate) fn unused_groups(&self, conn: &Connection) -> StorageResult<Vec<UnusedGroup>> {
        let mut prepared = conn.prepare_cached(sql::UNUSED_GROUPS)?;
        self.bind(&mut prepared)?;
        let mut rows = prepared.raw_query();
        let mut groups = Vec::new();
        while let Some(row) = rows.next()? {
            let cols = Cols::new(row);
            let visibility: Option<String> = cols.take()?;
            let language: String = cols.take()?;
            let test: bool = cols.take()?;
            let many = small_from(cols.take()?)?;
            groups.push((visibility_of(visibility.as_deref())?, language, test, many));
        }
        Ok(groups)
    }

    /// The callables with a complexity and how many have each.
    pub(crate) fn complexity(&self, conn: &Connection) -> StorageResult<Vec<(f64, u32)>> {
        let mut prepared = conn.prepare_cached(sql::COMPLEXITY_GROUPS)?;
        self.bind(&mut prepared)?;
        let mut rows = prepared.raw_query();
        let mut groups = Vec::new();
        while let Some(row) = rows.next()? {
            groups.push((row.get(0)?, small_from(row.get(1)?)?));
        }
        Ok(groups)
    }
}

/// How many symbols without usages there are for a visibility, a language and test code.
pub(crate) type UnusedGroup = (Option<Visibility>, String, bool, u32);

/// The columns of `STATISTICS_FACTS`, still raw.
struct FactRow {
    id: i64,
    name: String,
    qualified: String,
    parent: Option<i64>,
    path: String,
    kind: String,
    language: String,
    visibility: Option<String>,
    test: bool,
    overrides: bool,
    certain: u32,
    total: u32,
    measures: [Option<f64>; 5],
}

// codeskimr: allow CHK-102 maps a row of the facts query into its columns
fn read_fact(cols: &Cols<'_>) -> rusqlite::Result<FactRow> {
    Ok(FactRow {
        id: cols.take()?,
        name: cols.take()?,
        qualified: cols.take()?,
        parent: cols.take()?,
        path: cols.take()?,
        kind: cols.take()?,
        language: cols.take()?,
        visibility: cols.take()?,
        test: cols.take()?,
        overrides: cols.take()?,
        certain: cols.take()?,
        total: cols.take()?,
        measures: [
            cols.take()?,
            cols.take()?,
            cols.take()?,
            cols.take()?,
            cols.take()?,
        ],
    })
}

fn decode(row: FactRow) -> StorageResult<SymbolFacts> {
    let [complexity, loc, maintainability, efferent, afferent] = row.measures;
    Ok(SymbolFacts {
        id: SymbolId::new(id_from(row.id)?),
        name: SymbolName::new(row.name)?,
        qualified_name: QualifiedName::new(row.qualified)?,
        kind: label_from(&row.kind)?,
        parent: row.parent.map(id_from).transpose()?.map(SymbolId::new),
        visibility: visibility_of(row.visibility.as_deref())?,
        path: RelativePath::new(row.path)?,
        language: LanguageId::new(row.language)?,
        test: row.test,
        overrides: row.overrides,
        usage: UsageTally {
            certain: row.certain,
            total: row.total,
        },
        measures: Measures {
            complexity,
            loc,
            maintainability,
            efferent,
            afferent,
        },
    })
}

/// The facts of the symbols `ids` of a snapshot, in the order of `ids`.
pub(crate) fn facts_of(
    conn: &Connection,
    snapshot: SnapshotId,
    ids: &[i64],
) -> StorageResult<Vec<SymbolFacts>> {
    let list = id_list(
        ids.iter()
            .map(|id| id_from(*id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter(),
    )?;
    let raw = column_int(snapshot.get())?;
    let mut prepared = conn.prepare_cached(sql::STATISTICS_FACTS)?;
    let values: [(&str, &dyn ToSql); 2] = [(PARAM_SNAPSHOT, &raw), (PARAM_IDS, &list)];
    bind(&mut prepared, &values)?;
    let mut rows = prepared.raw_query();
    let mut found = std::collections::HashMap::new();
    while let Some(row) = rows.next()? {
        let fact = decode(read_fact(&Cols::new(row))?)?;
        found.insert(fact.id, fact);
    }
    Ok(ids
        .iter()
        .filter_map(|id| {
            id_from(*id)
                .ok()
                .and_then(|id| found.remove(&SymbolId::new(id)))
        })
        .collect())
}

/// Binds the parameters of `statement` that it has; the others are left out.
pub(crate) fn bind(
    statement: &mut Statement<'_>,
    values: &[(&str, &dyn ToSql)],
) -> StorageResult<()> {
    for (name, value) in values {
        if let Some(index) = statement.parameter_index(name)? {
            statement.raw_bind_parameter(index, value)?;
        }
    }
    Ok(())
}

/// The visibility a label stands for, `None` for no visibility.
pub(crate) fn visibility_of(label: Option<&str>) -> StorageResult<Option<Visibility>> {
    label.map(label_from).transpose()
}
