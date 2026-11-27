//! Which two retained scans a comparison reads, and whether they can be compared (ADR 0097).

use codeskimr_codes::ErrorCode;
use codeskimr_domain::SnapshotId;

use crate::error::CodedError;
use crate::ports::ScanEntry;

/// The positions in the history of the scans that wrote or confirmed `snapshot`, newest first, as
/// `(newest, oldest)`; `CMP-201` if retention does not keep it.
fn span(history: &[ScanEntry], snapshot: SnapshotId) -> Result<(usize, usize), CodedError> {
    let at = |reverse: bool| {
        let mut found = history
            .iter()
            .enumerate()
            .filter(|(_, it)| it.snapshot == snapshot);
        if reverse {
            found.next_back()
        } else {
            found.next()
        }
        .map(|(index, _)| index)
    };
    let newest = at(false).ok_or_else(|| CodedError::new(ErrorCode::SnapshotNotFound))?;
    Ok((newest, at(true).unwrap_or(newest)))
}

/// The scans to compare: `base` and `target` of the request, or the scan before the newest and
/// the newest. The history lists every scan, newest first, and scans that found nothing changed
/// share the snapshot they confirmed (ADR 0093). A snapshot named by the request stands for its
/// newest scan, except that a base that is the target snapshot stands for its oldest scan, so that
/// "nothing changed since the first scan of this snapshot" can be asked. The default compares the
/// two newest scans, which can be the same snapshot: the comparison is then empty.
///
/// # Errors
/// `CMP-201` if a snapshot is not retained or the default base does not exist, `CMP-101` if the
/// base snapshot is newer than the target snapshot.
pub(super) fn choose(
    history: &[ScanEntry],
    (base, target): (Option<SnapshotId>, Option<SnapshotId>),
) -> Result<(ScanEntry, ScanEntry), CodedError> {
    let target_at = target.map_or(Ok(usize::default()), |id| span(history, id).map(|it| it.0))?;
    let base_at = match base {
        None => target_at + 1,
        Some(id)
            if Some(id) == target || history.get(target_at).is_some_and(|it| it.snapshot == id) =>
        {
            span(history, id)?.1
        }
        Some(id) => span(history, id)?.0,
    };
    let (Some(older), Some(newer)) = (history.get(base_at), history.get(target_at)) else {
        return Err(CodedError::new(ErrorCode::SnapshotNotFound));
    };
    if older.snapshot > newer.snapshot {
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
