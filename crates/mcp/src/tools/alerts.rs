//! `get_alerts` (ADR 0068): the alerts of critical symbols that changed, were removed or lost their
//! place. It only reads and needs `read`; dismissing an alert is not offered to agents.

use codeskimr_application::access;
use codeskimr_application::annotations::{AlertEntry, AlertQuery, alerts};
use codeskimr_domain::{AlertKind, AlertState, Principal};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, optional_label, to_json};
use crate::args::AlertsArgs;
use crate::consts::{ARG_KIND, ARG_STATE, READ_TOOL_PERMISSION};
use crate::error::McpError;

/// One alert. A field that is empty is left out, so an answer stays short.
#[derive(Serialize)]
struct Item {
    id: u64,
    /// `changed`, `removed` or `orphaned`.
    kind: &'static str,
    /// `open` or `dismissed`.
    state: &'static str,
    symbol_kind: &'static str,
    /// Qualified name of the critical symbol.
    symbol: String,
    /// The reason the symbol was marked critical.
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    /// For `changed`: `content`, `signature`, `cc` or `loc`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    changes: Vec<&'static str>,
    /// For `removed`: references that remain; provisional, they are matched by name.
    #[serde(skip_serializing_if = "Option::is_none")]
    references: Option<u32>,
    /// For `orphaned`: the closest symbol of the same name.
    #[serde(skip_serializing_if = "Option::is_none")]
    closest: Option<String>,
    /// How often the symbol was seen changed while the alert was open.
    count: u32,
    /// Snapshot of the latest sighting.
    snapshot: u64,
    /// Seconds since the Unix epoch.
    first_seen: i64,
    last_seen: i64,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    /// Open alerts of the project, whatever the filter says.
    open: u32,
    /// Alerts that match the filter.
    total: u32,
    offset: u32,
    /// Pass it as `offset` to read the next page; absent on the last one.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_offset: Option<u32>,
    alerts: Vec<Item>,
}

fn item(entry: &AlertEntry) -> Item {
    let alert = &entry.alert;
    let target = &entry.annotation.target;
    Item {
        id: alert.id.get(),
        kind: alert.sighting.kind.label(),
        state: alert.state.label(),
        symbol_kind: target.kind.label(),
        symbol: target.name.as_str().to_owned(),
        reason: entry
            .annotation
            .content
            .text
            .as_ref()
            .map(ToString::to_string),
        changes: alert
            .sighting
            .changes
            .iter()
            .map(|change| change.label())
            .collect(),
        references: alert.sighting.references,
        closest: alert.sighting.closest.as_ref().map(ToString::to_string),
        count: alert.counter,
        snapshot: alert.sighting.snapshot.get(),
        first_seen: alert.first_seen.0,
        last_seen: alert.last_seen.0,
    }
}

impl Tools {
    /// The alerts of a project, newest sighting first; the open ones unless `state` says otherwise.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid argument, `MCP-201` and
    /// `MCP-203` for the project, or the code of a failing port.
    pub fn get_alerts(&self, principal: &Principal, args: &AlertsArgs) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let state = optional_label::<AlertState>(args.state.as_deref(), ARG_STATE)?
            .unwrap_or(AlertState::Open);
        let kind = optional_label::<AlertKind>(args.kind.as_deref(), ARG_KIND)?;
        let scope = self.scope(args.project)?;
        let query = AlertQuery {
            project: scope.project.id,
            state: Some(state),
            kinds: kind.into_iter().collect(),
        };
        let found = alerts(
            self.annotation_store(),
            &query,
            self.page_request(args.offset, args.limit),
        )?;
        to_json(&Answer {
            snapshot: scope.info,
            open: found.open,
            total: found.page.total,
            offset: found.page.offset,
            next_offset: found.page.next_offset(),
            alerts: found.page.items.iter().map(item).collect(),
        })
    }
}
