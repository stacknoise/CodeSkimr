//! Colours of the metrics: rates a value against the thresholds of the configuration.

use codeskimr_application::config::MetricThresholds;
use codeskimr_domain::MetricKind;

use super::consts::{RATING_GOOD, RATING_MEDIUM, RATING_POOR};

/// Rating of a value where less is better.
fn at_most(value: f64, good: f64, medium: f64) -> &'static str {
    if value <= good {
        RATING_GOOD
    } else if value <= medium {
        RATING_MEDIUM
    } else {
        RATING_POOR
    }
}

/// Rating of a value where more is better.
fn at_least(value: f64, good: f64, medium: f64) -> &'static str {
    if value >= good {
        RATING_GOOD
    } else if value >= medium {
        RATING_MEDIUM
    } else {
        RATING_POOR
    }
}

/// Rates the complexity of a callable.
pub fn complexity(value: f64, thresholds: &MetricThresholds) -> &'static str {
    at_most(
        value,
        f64::from(thresholds.cyclomatic_good_max),
        f64::from(thresholds.cyclomatic_medium_max),
    )
}

/// Rates a metric of a symbol; `None` for metrics that have no thresholds (sizes, coupling).
pub fn metric(kind: MetricKind, value: f64, thresholds: &MetricThresholds) -> Option<&'static str> {
    match kind {
        MetricKind::CyclomaticComplexity | MetricKind::CyclomaticComplexityMax => {
            Some(complexity(value, thresholds))
        }
        MetricKind::MaintainabilityIndex => Some(at_least(
            value,
            f64::from(thresholds.maintainability_good_min),
            f64::from(thresholds.maintainability_medium_min),
        )),
        MetricKind::Instability => Some(at_most(
            value,
            thresholds.instability_stable_max,
            thresholds.instability_medium_max,
        )),
        _ => None,
    }
}
