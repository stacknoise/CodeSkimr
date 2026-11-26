//! One scan state of a project.

use crate::consts::{
    DATE_SEPARATOR, DAYS_BEFORE_EPOCH, DAYS_OF_FIVE_MONTHS, DAYS_PER_CENTURY, DAYS_PER_ERA,
    DAYS_PER_FOUR_YEARS, DAYS_PER_YEAR, FEBRUARY, JANUARY_FROM_MARCH, LAST_DAY_OF_ERA, MARCH,
    MONTH_FACTOR, MONTH_OFFSET, MONTHS_FROM_MARCH, ORIGIN_BROWSER, ORIGIN_MCP, ORIGIN_TERMINAL,
    ORIGIN_WATCH, PAD, PART_DIGITS, SCAN_FULL, SCAN_INCREMENTAL, SECONDS_PER_DAY, YEAR_DIGITS,
    YEARS_PER_CENTURY, YEARS_PER_ERA, YEARS_PER_LEAP,
};
use crate::project::ProjectId;
use crate::vocabulary::{id_type, labeled_enum, text_type};

id_type!(
    /// Identifies a snapshot.
    SnapshotId
);
text_type!(
    /// Git commit hash a snapshot was taken at.
    CommitHash
);

/// Seconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

/// `value` as text, filled with zeros in front up to `width` digits.
fn padded(value: i64, width: usize) -> String {
    let digits = value.to_string();
    [PAD.repeat(width.saturating_sub(digits.len())), digits].concat()
}

impl Timestamp {
    /// The calendar day in UTC as `YYYY-MM-DD`, the form the held state of the rules stores.
    #[must_use]
    pub fn iso_date(self) -> String {
        let shifted = self.0.div_euclid(SECONDS_PER_DAY) + DAYS_BEFORE_EPOCH;
        let era = shifted.div_euclid(DAYS_PER_ERA);
        let day_of_era = shifted - era * DAYS_PER_ERA;
        let year_of_era = (day_of_era - day_of_era / DAYS_PER_FOUR_YEARS
            + day_of_era / DAYS_PER_CENTURY
            - day_of_era / LAST_DAY_OF_ERA)
            / DAYS_PER_YEAR;
        let day_of_year = day_of_era
            - (DAYS_PER_YEAR * year_of_era + year_of_era / YEARS_PER_LEAP
                - year_of_era / YEARS_PER_CENTURY);
        let month_from_march = (MONTH_FACTOR * day_of_year + MONTH_OFFSET) / DAYS_OF_FIVE_MONTHS;
        let day = day_of_year
            - (DAYS_OF_FIVE_MONTHS * month_from_march + MONTH_OFFSET) / MONTH_FACTOR
            + 1;
        let month = if month_from_march < MONTHS_FROM_MARCH {
            month_from_march + MARCH
        } else {
            month_from_march - JANUARY_FROM_MARCH
        };
        let year = year_of_era + era * YEARS_PER_ERA + i64::from(month <= FEBRUARY);
        [
            padded(year, YEAR_DIGITS),
            padded(month, PART_DIGITS),
            padded(day, PART_DIGITS),
        ]
        .join(DATE_SEPARATOR)
    }
}

labeled_enum!(
    /// How much of the project a scan processed.
    ScanMode {
        /// Every file was processed.
        Full => SCAN_FULL,
        /// Only files with a changed hash were processed.
        Incremental => SCAN_INCREMENTAL,
    }
);

labeled_enum!(
    /// Which channel started a scan. The server decides it from the channel that received the
    /// request, never from a field of the request (ADR 0072, decision 2).
    ScanOrigin {
        /// `codeskimr scan` at the terminal of the owner.
        Terminal => ORIGIN_TERMINAL,
        /// A command of the HTTP layer of `codeskimr serve`, the page in the browser.
        Browser => ORIGIN_BROWSER,
        /// A later scan of `codeskimr scan --watch`.
        Watch => ORIGIN_WATCH,
        /// A rescan through MCP.
        Mcp => ORIGIN_MCP,
    }
);

/// Totals of a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SnapshotStats {
    /// Number of files.
    pub files: u32,
    /// Number of symbols.
    pub symbols: u32,
    /// Number of references.
    pub references: u32,
}

/// One scan state of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Identity.
    pub id: SnapshotId,
    /// Owning project.
    pub project: ProjectId,
    /// Commit, if the source is a Git repository.
    pub commit: Option<CommitHash>,
    /// Creation time.
    pub created: Timestamp,
    /// Full or incremental scan.
    pub mode: ScanMode,
    /// Totals.
    pub stats: SnapshotStats,
}
