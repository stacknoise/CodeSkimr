//! The facts the statistics screen ranks (ADR 0081, ADR 0091; adapter: `codeskimr_storage`).

use codeskimr_domain::{Assessment, RankingQuery, SnapshotId, SymbolFacts};

use crate::error::CodedError;

/// The settings the statistics are judged by.
#[derive(Debug, Clone, Copy)]
pub struct StatisticsSetup<'a> {
    /// `conventions.test_folders`: names of folders and modules that hold test code.
    pub test_names: &'a [String],
    /// `ui.approximate_languages`: languages whose usages are an approximation.
    pub approximate: &'a [String],
}

/// A ranking that is asked for.
#[derive(Debug, Clone, Copy)]
pub struct RankingRequest<'a> {
    /// The snapshot to rank.
    pub snapshot: SnapshotId,
    /// What to rank.
    pub query: &'a RankingQuery,
    /// For the ranking of unused symbols: only rows with this assessment; all if `None`.
    pub assessment: Option<Assessment>,
    /// Most rows to return.
    pub limit: usize,
    /// The settings the ranking is judged by.
    pub setup: StatisticsSetup<'a>,
}

/// The first rows of a ranking and how many rows it holds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RankedRows {
    /// The rows, best first.
    pub rows: Vec<SymbolFacts>,
    /// Number of symbols the ranking holds.
    pub total: usize,
}

/// How many symbols without usages there are per assessment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnusedCounts {
    /// Not public, in a language with exact usages: very likely unused.
    pub likely: u32,
    /// Public: code outside the project may use it.
    pub public: u32,
    /// In a language whose usages are an approximation.
    pub approximate: u32,
    /// Test code without usages that the answer leaves out; zero if test code counts.
    pub tests_hidden: u32,
}

/// What the cards of the overview sum up over the whole snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatisticsSummary {
    /// Number of types and callables the rankings hold.
    pub ranked: u32,
    /// References that are certain, of all symbols.
    pub certain_references: u64,
    /// References that are only probable, of all symbols.
    pub probable_references: u64,
    /// The callables per cyclomatic complexity, by rising complexity.
    pub histogram: Vec<(u32, u32)>,
}

/// Answers the questions of the statistics screen from stored numbers (ADR 0091). They are counted
/// once per snapshot, right after the scan or the import of level 2 has written it (`prepare`),
/// and at the first question if that has not happened (a scan by another program, an older
/// database); later questions and a restart of the program read them from the database.
pub trait CodeStatistics: Send + Sync {
    /// Every symbol of the snapshot with its usage counts and measured values, counted from the
    /// references now: the calculation the stored numbers must equal. A symbol is marked as test
    /// code if its file holds a reference of kind `test`, which only test files do. A snapshot
    /// that is not stored has no symbols.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn symbol_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFacts>, CodedError>;

    /// Counts the numbers of the snapshot now if they are not counted yet (or were counted for
    /// other test folders), so that the first question finds them. A second caller that comes
    /// while they are counted waits for them.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn prepare(&self, snapshot: SnapshotId, setup: StatisticsSetup<'_>) -> Result<(), CodedError>;

    /// The first rows of a ranking, test code marked by the files and by the test folders.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn ranked(&self, request: &RankingRequest<'_>) -> Result<RankedRows, CodedError>;

    /// How many symbols without usages the ranking `query` holds per assessment; `query.ranking`
    /// is ignored.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn unused_counts(
        &self,
        snapshot: SnapshotId,
        query: &RankingQuery,
        setup: StatisticsSetup<'_>,
    ) -> Result<UnusedCounts, CodedError>;

    /// The sums of the overview; test code counts in the histogram and in `ranked` only with
    /// `include_tests`.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn summary(
        &self,
        snapshot: SnapshotId,
        include_tests: bool,
        setup: StatisticsSetup<'_>,
    ) -> Result<StatisticsSummary, CodedError>;
}
