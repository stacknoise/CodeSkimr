//! What a scan says about itself: its level and why level 2 did not check a language (ADR 0072).

use crate::consts::{
    LEVEL_ONE, LEVEL_TWO, OUTCOME_CHECKED, OUTCOME_FAILED, OUTCOME_SKIPPED, SKIP_BUSY,
    SKIP_CANCELLED, SKIP_NOT_CONFIRMED, SKIP_NOT_PERMITTED, SKIP_PAUSED, SKIP_STALE,
    SKIP_TOOLS_UNAVAILABLE, STAGE_LEVEL2_IMPORTING, STAGE_LEVEL2_RUNNING,
};
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// The level a scan reached: level 2 only if an indexer checked at least one language.
    ScanLevel {
        /// Name resolution only.
        One => LEVEL_ONE,
        /// An indexer checked at least one language.
        Two => LEVEL_TWO,
    }
);

labeled_enum!(
    /// Why level 2 did not check a language. A key, never a text: the screens translate it.
    SkipReason {
        /// The caller may not start indexers.
        NotPermitted => SKIP_NOT_PERMITTED,
        /// Level 2 is not enabled or not confirmed for the project.
        NotConfirmed => SKIP_NOT_CONFIRMED,
        /// The indexer or its toolchain is missing or does not fit.
        ToolsUnavailable => SKIP_TOOLS_UNAVAILABLE,
        /// Every file changed since the snapshot, so no index fits.
        Stale => SKIP_STALE,
        /// The run was cancelled before the language was stored.
        Cancelled => SKIP_CANCELLED,
        /// Another level 2 run held the place.
        Busy => SKIP_BUSY,
        /// A run of the same project ended a moment ago.
        Paused => SKIP_PAUSED,
    }
);

labeled_enum!(
    /// What level 2 did for one language.
    OutcomeKind {
        /// An indexer checked the language.
        Checked => OUTCOME_CHECKED,
        /// Level 2 did not check the language, for a reason.
        Skipped => OUTCOME_SKIPPED,
        /// The run failed with an error code.
        Failed => OUTCOME_FAILED,
    }
);

labeled_enum!(
    /// What a level 2 run is doing for a language right now.
    Level2Stage {
        /// The indexer runs the build tools of the project.
        Running => STAGE_LEVEL2_RUNNING,
        /// The index is read and mapped onto the symbols.
        Importing => STAGE_LEVEL2_IMPORTING,
    }
);
