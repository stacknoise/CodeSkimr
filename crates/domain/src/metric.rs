//! Measured values per symbol.

use crate::ModelError;
use crate::consts::{
    METRIC_CA, METRIC_CC, METRIC_CC_MAX, METRIC_CE, METRIC_INSTABILITY, METRIC_LOC, METRIC_MI,
};
use crate::symbol::SymbolId;
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// What a metric measures.
    MetricKind {
        /// Lines of code.
        Loc => METRIC_LOC,
        /// Cyclomatic complexity: of a callable, the sum over the callables below a type or module.
        CyclomaticComplexity => METRIC_CC,
        /// Highest cyclomatic complexity of a callable below a type or module (ADR 0015).
        CyclomaticComplexityMax => METRIC_CC_MAX,
        /// Maintainability index.
        MaintainabilityIndex => METRIC_MI,
        /// Efferent coupling: how many others it depends on.
        EfferentCoupling => METRIC_CE,
        /// Afferent coupling: how many others depend on it.
        AfferentCoupling => METRIC_CA,
        /// Instability, `ce / (ca + ce)`.
        Instability => METRIC_INSTABILITY,
    }
);

impl MetricKind {
    /// Whether the value depends on references across files and therefore belongs to one
    /// snapshot, not to the symbol for all snapshots (ADR 0037): coupling and instability.
    #[must_use]
    pub const fn is_per_snapshot(self) -> bool {
        matches!(
            self,
            Self::EfferentCoupling | Self::AfferentCoupling | Self::Instability
        )
    }
}

/// A finite measured number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MetricValue(f64);

impl MetricValue {
    /// Wraps a measured number.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::NotFinite`] for NaN and infinity.
    pub fn new(value: f64) -> Result<Self, ModelError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ModelError::NotFinite)
        }
    }

    /// The number.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// A measured value of one symbol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metric {
    /// Measured symbol.
    pub symbol: SymbolId,
    /// What was measured.
    pub kind: MetricKind,
    /// The value.
    pub value: MetricValue,
}
