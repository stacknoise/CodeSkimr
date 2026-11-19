//! `codeskimr annotations` and `codeskimr alerts` (ADR 0068): the commands of the registry run as
//! the local owner, and their answer is printed as text or, with `--json`, as the JSON the REST
//! route returns.

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::registry::{Command, Registry};
use codeskimr_cli::{
    AddAnnotation, AlertsAction, AnnotationFields, AnnotationsAction, ById, Clears, DismissAlerts,
    GlobalOptions, ListAlerts, ListAnnotations, PageArgs, SymbolArg, UpdateAnnotation,
};
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_ALERT_LINE, CLI_ALERT_NONE, CLI_ALERTS_DISMISSED, CLI_ALERTS_PAGE, CLI_ANNOTATION_LINE,
    CLI_ANNOTATION_NONE, CLI_ANNOTATION_REMOVED, CLI_ANNOTATION_SAVED, CLI_ANNOTATIONS_PAGE,
};
use codeskimr_i18n::{FluentArgs, FluentValue, Messages};
use serde::Serialize;

use crate::consts::{CLI_KEPT_CONTROLS, CLI_LINE_BREAK, CLI_LIST_SEPARATOR, note_arg};
use crate::locations::Locations;
use crate::ui_commands::{
    AlertListInput, AlertListOut, AlertOut, AlertsDismiss, AlertsDismissAll, AlertsList,
    AnnotationOut, AnnotationsDelete, AnnotationsList, AnnotationsUpdate, AnyKind, Create,
    CreateInput, DeleteInput, DismissAllInput, DismissInput, Dismissed, ListInput, Nullable,
    PageOut, TargetIn, UpdateInput, annotation_ports, register_annotations,
};
use crate::wiring::open_store;
use crate::{config_files, export, principal};

/// What a run needs besides its action: the global options, the folders and the texts.
pub struct Surroundings<'a> {
    /// Global options of the command line.
    pub global: &'a GlobalOptions,
    /// Folders of the installation.
    pub locations: &'a Locations,
    /// The texts.
    pub messages: &'a Messages,
    /// The language to print in.
    pub language: &'a str,
}

/// What the commands run over: the registry with the annotation commands and the project the
/// caller named.
struct Session {
    registry: Registry,
    project: u64,
}

impl Session {
    fn open(project: &str, around: &Surroundings<'_>) -> Result<Self, CodedError> {
        let config = config_files::load(around.locations, around.global, None)?;
        let store = open_store(around.locations, &config)?;
        let project = export::resolve(project, &store)?;
        let ports = annotation_ports(&store, &config, &EventBus::new())?;
        let mut registry = Registry::new();
        register_annotations(&mut registry, &ports)?;
        Ok(Self { registry, project })
    }

    fn run<C: Command>(&self, input: C::Input) -> Result<C::Output, CodedError> {
        self.registry.execute::<C>(&principal::resolve(), input)
    }
}

fn target(symbol: &SymbolArg) -> TargetIn {
    TargetIn {
        kind: symbol.kind.clone(),
        name: symbol.name.clone(),
        parameters: symbol.parameters.clone(),
    }
}

fn nullable<T>(value: Option<T>, clear: bool) -> Nullable<T> {
    match (value, clear) {
        (Some(value), _) => Nullable::Set(value),
        (None, true) => Nullable::Clear,
        (None, false) => Nullable::Keep,
    }
}

fn json_of<T: Serialize>(answer: &T) -> Result<String, CodedError> {
    serde_json::to_string_pretty(answer)
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))
}

/// The JSON of an answer, or the text that `text` makes.
fn printed<T: Serialize>(
    answer: &T,
    json: bool,
    text: impl FnOnce() -> Result<String, CodedError>,
) -> Result<String, CodedError> {
    if json { json_of(answer) } else { text() }
}

/// The text without control characters (ANSI and OSC escapes, C0 and C1) except tab and line
/// break: a note is user input, and a terminal must not act on it.
pub(crate) fn printable(text: &str) -> String {
    text.chars()
        .filter(|found| !found.is_control() || CLI_KEPT_CONTROLS.contains(found))
        .collect()
}

fn say(around: &Surroundings<'_>, key: &str, args: &FluentArgs<'_>) -> Result<String, CodedError> {
    around
        .messages
        .format(around.language, key, Some(args))
        .map(|text| printable(&text))
        .map_err(CodedError::caused_by)
}

fn number(value: u32) -> FluentValue<'static> {
    FluentValue::from(value)
}

/// What an annotation says, as one short text: status, title and text.
fn summary_of(annotation: &AnnotationOut) -> String {
    [
        annotation.status.map(str::to_owned),
        annotation.title.clone(),
        annotation.text.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(CLI_LIST_SEPARATOR)
}

fn annotation_line(
    around: &Surroundings<'_>,
    annotation: &AnnotationOut,
) -> Result<String, CodedError> {
    let summary = summary_of(annotation);
    let mut args = FluentArgs::new();
    args.set(note_arg::ID, FluentValue::from(annotation.id));
    args.set(note_arg::KIND, annotation.kind);
    args.set(note_arg::NAME, annotation.target.name.clone());
    args.set(note_arg::PARTS, number(u32::from(!summary.is_empty())));
    args.set(note_arg::SUMMARY, summary);
    say(around, CLI_ANNOTATION_LINE, &args)
}

/// The lines of a page and, if there is more, the hint for the next page.
fn page_text<T>(
    page: &PageOut<T>,
    none: String,
    line: impl Fn(&T) -> Result<String, CodedError>,
    more: impl Fn(u32) -> Result<String, CodedError>,
) -> Result<String, CodedError> {
    let mut lines = page.items.iter().map(line).collect::<Result<Vec<_>, _>>()?;
    if lines.is_empty() {
        lines.push(none);
    }
    if let Some(next) = page.next_offset {
        lines.push(more(next)?);
    }
    Ok(lines.join(CLI_LINE_BREAK))
}

fn annotations_text(
    page: &PageOut<AnnotationOut>,
    around: &Surroundings<'_>,
) -> Result<String, CodedError> {
    let none = say(around, CLI_ANNOTATION_NONE, &FluentArgs::new())?;
    page_text(
        page,
        none,
        |annotation| annotation_line(around, annotation),
        |next| {
            let mut args = FluentArgs::new();
            args.set(note_arg::SHOWN, FluentValue::from(page.items.len()));
            args.set(note_arg::TOTAL, number(page.total));
            args.set(note_arg::NEXT, number(next));
            say(around, CLI_ANNOTATIONS_PAGE, &args)
        },
    )
}

fn saved_text(annotation: &AnnotationOut, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let mut args = FluentArgs::new();
    args.set(note_arg::ID, FluentValue::from(annotation.id));
    args.set(note_arg::NAME, annotation.target.name.clone());
    say(around, CLI_ANNOTATION_SAVED, &args)
}

fn create_input(project: u64, add: &AddAnnotation) -> CreateInput {
    let AnnotationFields {
        title,
        text,
        status,
        due,
        assignee,
    } = add.fields.clone();
    CreateInput {
        project,
        target: target(&add.symbol),
        kind: Some(add.kind.clone()),
        title,
        text,
        status,
        due,
        assignee,
    }
}

fn update_input(project: u64, update: &UpdateAnnotation) -> UpdateInput {
    let fields = &update.fields;
    let Clears {
        text,
        due,
        assignee,
    } = update.clears;
    UpdateInput {
        project,
        id: update.id,
        target: update.symbol.as_ref().map(target),
        title: fields.title.clone(),
        text: nullable(fields.text.clone(), text),
        status: fields.status.clone(),
        due: nullable(fields.due, due),
        assignee: nullable(fields.assignee.clone(), assignee),
    }
}

fn list_input(project: u64, list: &ListAnnotations) -> ListInput {
    let PageArgs { offset, limit } = list.page;
    ListInput {
        project,
        symbol: list.symbol.as_ref().map(target),
        kind: list.kind.clone(),
        status: list.status.clone(),
        without_target: None,
        offset,
        limit,
    }
}

fn list_annotations(
    list: &ListAnnotations,
    around: &Surroundings<'_>,
) -> Result<String, CodedError> {
    let session = Session::open(&list.project, around)?;
    let listed = session.run::<AnnotationsList>(list_input(session.project, list))?;
    printed(&listed, list.json, || annotations_text(&listed, around))
}

fn add_annotation(add: &AddAnnotation, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let session = Session::open(&add.project, around)?;
    let added = session.run::<Create<AnyKind>>(create_input(session.project, add))?;
    printed(&added, add.json, || saved_text(&added, around))
}

// codeskimr: allow CHK-102 runs the update command of an annotation and prints its result
fn update_annotation(
    update: &UpdateAnnotation,
    around: &Surroundings<'_>,
) -> Result<String, CodedError> {
    let session = Session::open(&update.project, around)?;
    let updated = session.run::<AnnotationsUpdate>(update_input(session.project, update))?;
    printed(&updated, update.json, || saved_text(&updated, around))
}

fn remove_annotation(remove: &ById, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let session = Session::open(&remove.project, around)?;
    let removed = session.run::<AnnotationsDelete>(DeleteInput {
        project: session.project,
        id: remove.id,
    })?;
    printed(&removed, remove.json, || {
        let mut args = FluentArgs::new();
        args.set(note_arg::ID, FluentValue::from(removed.id));
        say(around, CLI_ANNOTATION_REMOVED, &args)
    })
}

/// Runs `codeskimr annotations ...` and returns what to print.
///
/// # Errors
/// The code of the command that failed, `PRJ-201` for an unknown project, `I18N-201` for a missing
/// text.
pub fn run_annotations(
    action: &AnnotationsAction,
    around: &Surroundings<'_>,
) -> Result<String, CodedError> {
    match action {
        AnnotationsAction::List(list) => list_annotations(list, around),
        AnnotationsAction::Add(add) => add_annotation(add, around),
        AnnotationsAction::Update(update) => update_annotation(update, around),
        AnnotationsAction::Remove(remove) => remove_annotation(remove, around),
    }
}

/// The detail of an alert: what changed, how many references remain, or the closest symbol.
fn alert_detail(alert: &AlertOut) -> String {
    let mut parts: Vec<String> = alert
        .changes
        .iter()
        .map(|change| (*change).to_owned())
        .collect();
    parts.extend(alert.references.map(|count| count.to_string()));
    parts.extend(alert.closest.clone());
    parts.join(CLI_LIST_SEPARATOR)
}

// codeskimr: allow CHK-102 puts the fields of an alert into the arguments of its text
fn alert_line(around: &Surroundings<'_>, alert: &AlertOut) -> Result<String, CodedError> {
    let detail = alert_detail(alert);
    let mut args = FluentArgs::new();
    args.set(note_arg::ID, FluentValue::from(alert.id));
    args.set(note_arg::STATE, alert.state);
    args.set(note_arg::KIND, alert.kind);
    args.set(note_arg::NAME, alert.target.name.clone());
    args.set(note_arg::PARTS, number(u32::from(!detail.is_empty())));
    args.set(note_arg::DETAIL, detail);
    say(around, CLI_ALERT_LINE, &args)
}

fn alerts_text(listed: &AlertListOut, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let none = say(around, CLI_ALERT_NONE, &FluentArgs::new())?;
    page_text(
        &listed.page,
        none,
        |alert| alert_line(around, alert),
        |next| {
            let mut args = FluentArgs::new();
            args.set(note_arg::SHOWN, FluentValue::from(listed.page.items.len()));
            args.set(note_arg::TOTAL, number(listed.page.total));
            args.set(note_arg::OPEN, number(listed.open));
            args.set(note_arg::NEXT, number(next));
            say(around, CLI_ALERTS_PAGE, &args)
        },
    )
}

fn dismissed_text(done: Dismissed, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let mut args = FluentArgs::new();
    args.set(note_arg::DISMISSED, number(done.dismissed));
    args.set(note_arg::OPEN, number(done.open));
    say(around, CLI_ALERTS_DISMISSED, &args)
}

fn list_alerts(list: &ListAlerts, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let session = Session::open(&list.project, around)?;
    let PageArgs { offset, limit } = list.page;
    let listed = session.run::<AlertsList>(AlertListInput {
        project: session.project,
        state: list.state.clone(),
        kinds: list.kinds.clone(),
        offset,
        limit,
    })?;
    printed(&listed, list.json, || alerts_text(&listed, around))
}

fn dismiss_alert(dismiss: &ById, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let session = Session::open(&dismiss.project, around)?;
    let done = session.run::<AlertsDismiss>(DismissInput {
        project: session.project,
        alert: dismiss.id,
    })?;
    printed(&done, dismiss.json, || dismissed_text(done, around))
}

// codeskimr: allow CHK-102 runs the dismiss-all command of alerts and prints its result
fn dismiss_alerts(all: &DismissAlerts, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let session = Session::open(&all.project, around)?;
    let done = session.run::<AlertsDismissAll>(DismissAllInput {
        project: session.project,
        kinds: all.kinds.clone(),
        confirm: all.confirm,
    })?;
    printed(&done, all.json, || dismissed_text(done, around))
}

/// Runs `codeskimr alerts ...` and returns what to print.
///
/// # Errors
/// The code of the command that failed, `API-101` for `dismiss-all` without `--yes`, `PRJ-201` for
/// an unknown project, `I18N-201` for a missing text.
pub fn run_alerts(action: &AlertsAction, around: &Surroundings<'_>) -> Result<String, CodedError> {
    match action {
        AlertsAction::List(list) => list_alerts(list, around),
        AlertsAction::Dismiss(dismiss) => dismiss_alert(dismiss, around),
        AlertsAction::DismissAll(all) => dismiss_alerts(all, around),
    }
}
