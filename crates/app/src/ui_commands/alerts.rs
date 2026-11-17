//! The commands of the alerts (ADR 0068): list, dismiss one and dismiss all of a filter.

use std::sync::Arc;

use codeskimr_application::annotations::{AlertEntry, AlertQuery, alerts};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::i18n;
use codeskimr_domain::{AlertId, AlertKind, Permission, ProjectId};
use serde::{Deserialize, Serialize};

use super::annotation_wire::{AlertOut, PageOut, invalid, label, optional_label};
use super::annotations::{AnnotationPorts, read_descriptor};
use super::consts::{ALERTS_DISMISS, ALERTS_DISMISS_ALL, ALERTS_LIST};

/// The input of `alerts.list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AlertListInput {
    /// Id of the project.
    pub project: u64,
    /// Only alerts in this state: `open` or `dismissed`.
    #[serde(default)]
    pub state: Option<String>,
    /// Only alerts of these kinds (`changed`, `removed`, `orphaned`); all kinds if empty.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Rows to skip.
    #[serde(default)]
    pub offset: Option<u32>,
    /// Rows to return, at most `annotations.max_page_size`.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// A page of alerts and the number of open alerts of the project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AlertListOut {
    /// The page.
    #[serde(flatten)]
    pub page: PageOut<AlertOut>,
    /// Open alerts of the whole project, whatever the filter says: the number on the bell.
    pub open: u32,
}

fn kinds(labels: &[String]) -> Result<Vec<AlertKind>, CodedError> {
    labels.iter().map(|text| label(text)).collect()
}

/// Lists the alerts of a project.
pub struct AlertsList {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AlertsList {
    type Input = AlertListInput;
    type Output = AlertListOut;

    const DESCRIPTOR: CommandDescriptor = read_descriptor(
        ALERTS_LIST,
        (
            i18n::COMMAND_ALERTS_LIST_TITLE,
            i18n::COMMAND_ALERTS_LIST_DESCRIPTION,
        ),
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<AlertListOut, CodedError> {
        let ports = &self.ports;
        let input = &request.input;
        let query = AlertQuery {
            project: ports.project(input.project)?,
            state: optional_label(input.state.as_deref())?,
            kinds: kinds(&input.kinds)?,
        };
        let found = alerts(
            ports.store.as_ref(),
            &query,
            ports.page(input.offset, input.limit),
        )?;
        Ok(AlertListOut {
            page: PageOut::of(&found.page, |entry: &AlertEntry| AlertOut::from(entry)),
            open: found.open,
        })
    }
}

/// The input of `alerts.dismiss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct DismissInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the alert.
    pub alert: u64,
}

/// What a dismissal answers: the number of open alerts that remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Dismissed {
    /// How many alerts were dismissed.
    pub dismissed: u32,
    /// Open alerts of the project now.
    pub open: u32,
}

/// Dismisses one alert.
pub struct AlertsDismiss {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AlertsDismiss {
    type Input = DismissInput;
    type Output = Dismissed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(ALERTS_DISMISS),
        title_key: i18n::COMMAND_ALERTS_DISMISS_TITLE,
        description_key: i18n::COMMAND_ALERTS_DISMISS_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Annotate,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<Dismissed, CodedError> {
        let input = request.input;
        let project = ProjectId::new(input.project);
        self.ports
            .writes
            .dismiss(project, AlertId::new(input.alert))?;
        Ok(Dismissed {
            dismissed: 1,
            open: self.ports.open_alerts(project)?,
        })
    }
}

/// The input of `alerts.dismiss_all`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DismissAllInput {
    /// Id of the project.
    pub project: u64,
    /// Only alerts of these kinds; all kinds if empty.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Must be `true`: dismissing many alerts at once is confirmed explicitly, not by default.
    #[serde(default)]
    pub confirm: bool,
}

/// Dismisses every open alert of a filter.
pub struct AlertsDismissAll {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AlertsDismissAll {
    type Input = DismissAllInput;
    type Output = Dismissed;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(ALERTS_DISMISS_ALL),
        title_key: i18n::COMMAND_ALERTS_DISMISS_ALL_TITLE,
        description_key: i18n::COMMAND_ALERTS_DISMISS_ALL_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Annotate,
    };

    /// # Errors
    /// `API-101` without `confirm: true`; nothing is dismissed then.
    fn execute(&self, request: Request<'_, Self>) -> Result<Dismissed, CodedError> {
        let input = &request.input;
        if !input.confirm {
            return Err(invalid());
        }
        let project = ProjectId::new(input.project);
        let query = AlertQuery {
            project,
            state: None,
            kinds: kinds(&input.kinds)?,
        };
        let dismissed = self.ports.writes.dismiss_all(&query)?;
        Ok(Dismissed {
            dismissed,
            open: self.ports.open_alerts(project)?,
        })
    }
}
