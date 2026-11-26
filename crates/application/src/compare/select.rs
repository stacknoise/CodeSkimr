//! Which two retained scans a comparison reads, and whether they can be compared (ADR 0097).

use codeskimr_codes::ErrorCode;
use codeskimr_domain::SnapshotId;

use crate::error::CodedError;
use crate::ports::ScanEntry;

/// The position of a snapshot in the history, `CMP-201` if retention does not keep it.
fn position(history: &[ScanEntry], snapshot: SnapshotId) -> Result<usize, CodedError> {
    history
        .iter()
        .position(|entry| entry.snapshot == snapshot)
        .ok_or_else(|| CodedError::new(ErrorCode::SnapshotNotFound))
}

/// The scans to compare: `base` and `target` of the request, or the scan before the newest and
/// the newest. The history is newest first, so the older a scan, the larger its position.
///
/// # Errors
/// `CMP-201` if a snapshot is not retained or the default base does not exist, `CMP-101` if the
/// base is not older than the target.
pub(super) fn choose(
    history: &[ScanEntry],
    (base, target): (Option<SnapshotId>, Option<SnapshotId>),
) -> Result<(ScanEntry, ScanEntry), CodedError> {
    let target_at = target.map_or(Ok(usize::default()), |id| position(history, id))?;
    let base_at = base.map_or(Ok(target_at + 1), |id| position(history, id))?;
    let (Some(older), Some(newer)) = (history.get(base_at), history.get(target_at)) else {
        return Err(CodedError::new(ErrorCode::SnapshotNotFound));
    };
    if base_at <= target_at {
        return Err(CodedError::new(ErrorCode::ComparisonOrderInvalid));
    }
    Ok((older.clone(), newer.clone()))
}

/// Whether two scans named their symbols by the same rules. A scan that has no record (written
/// before the rules were stored) only matches another one without.
///
/// # Errors
/// `CMP-301` if the rules differ.
pub(super) fn same_rules(base: Option<&str>, target: Option<&str>) -> Result<(), CodedError> {
    if base == target {
        Ok(())
    } else {
        Err(CodedError::new(ErrorCode::NamingRulesDiffer))
    }
}
