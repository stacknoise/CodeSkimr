//! The wiring of the comparison of two scans (ADR 0097): the store is every port it reads.

use std::sync::Arc;

use codeskimr_application::compare::{CompareLimits, ComparePorts, MetricRules, ScanComparer};
use codeskimr_application::config::Config;
use codeskimr_storage::SqliteStore;

/// The use case over `store`, with the limits of `compare.*` of the configuration.
pub(crate) fn comparer(store: &Arc<SqliteStore>, config: &Config) -> ScanComparer {
    ScanComparer::new(
        ComparePorts {
            index: store.clone(),
            source: store.clone(),
            usages: store.clone(),
            annotations: store.clone(),
        },
        CompareLimits::from(&config.compare),
        MetricRules::from_config(config),
    )
}
