//! `codeskimr scope` (ADR 0101): the commands `scope.get` and `scope.set` of the registry run as
//! the local owner, and their answer is printed as text or, with `--json`, as the JSON the REST
//! route returns.

use codeskimr_application::error::CodedError;
use codeskimr_application::registry::Registry;
use codeskimr_cli::ScopeArgs;
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_SCOPE_FOLDER, CLI_SCOPE_HEADING, CLI_SCOPE_LANGUAGE, CLI_SCOPE_MORE, CLI_SCOPE_SAVED,
    CLI_SCOPE_UNCHANGED,
};
use codeskimr_i18n::{FluentArgs, FluentValue};

use crate::annotations_cli::{Surroundings, say};
use crate::consts::{
    CLI_LINE_BREAK, CLI_SCOPE_COVERED, CLI_SCOPE_LEFT_OUT, CLI_SCOPE_UNCOUNTED, scope_arg,
};
use crate::ui_commands::{
    ScopeAnswer, ScopeGet, ScopeGetInput, ScopeSet, ScopeSetInput, register_scope,
};
use crate::wiring::open_store;
use crate::{config_files, export, principal};

/// What the scope is after the changes of `args`: the lists that `scope.set` stores.
fn changed(args: &ScopeArgs, now: &ScopeAnswer) -> (Vec<String>, Vec<String>) {
    let (mut languages, mut folders) = if args.reset {
        (Vec::new(), Vec::new())
    } else {
        (now.scope.languages.clone(), now.scope.folders.clone())
    };
    languages.extend(args.without_languages.iter().cloned());
    folders.extend(args.without_folders.iter().cloned());
    languages.retain(|language| !args.with_languages.contains(language));
    folders.retain(|folder| !args.with_folders.contains(folder));
    (languages, folders)
}

fn state(covered: bool) -> &'static str {
    if covered {
        CLI_SCOPE_COVERED
    } else {
        CLI_SCOPE_LEFT_OUT
    }
}

fn files_text(files: Option<u64>) -> String {
    files.map_or_else(|| CLI_SCOPE_UNCOUNTED.to_owned(), |count| count.to_string())
}

fn lines_of(answer: &ScopeAnswer, around: &Surroundings<'_>) -> Result<Vec<String>, CodedError> {
    let mut heading = FluentArgs::new();
    heading.set(scope_arg::PROJECT, FluentValue::from(answer.project));
    let mut lines = vec![say(around, CLI_SCOPE_HEADING, &heading)?];
    for language in &answer.languages {
        let mut args = FluentArgs::new();
        args.set(scope_arg::STATE, state(language.covered));
        args.set(scope_arg::LANGUAGE, language.language.clone());
        args.set(scope_arg::FILES, files_text(language.files));
        lines.push(say(around, CLI_SCOPE_LANGUAGE, &args)?);
    }
    for folder in &answer.folders {
        let mut args = FluentArgs::new();
        args.set(scope_arg::STATE, state(folder.covered));
        args.set(scope_arg::PATH, folder.path.clone());
        args.set(scope_arg::FILES, files_text(folder.files));
        lines.push(say(around, CLI_SCOPE_FOLDER, &args)?);
    }
    if answer.folders_more > 0 {
        let mut args = FluentArgs::new();
        args.set(scope_arg::COUNT, FluentValue::from(answer.folders_more));
        lines.push(say(around, CLI_SCOPE_MORE, &args)?);
    }
    Ok(lines)
}

/// Shows the scope of a project, after the changes that `args` asks for.
///
/// # Errors
/// `PRJ-201` for an unknown project, `PRJ-103` for an invalid scope, `SRC-201` if the folder
/// cannot be read, any coded failure of the configuration or the storage.
pub fn run_scope(args: &ScopeArgs, around: &Surroundings<'_>) -> Result<String, CodedError> {
    let config = config_files::load(around.locations, around.global, None)?;
    let store = open_store(around.locations, &config)?;
    let project = export::resolve(&args.project, &store)?;
    let mut registry = Registry::new();
    register_scope(&mut registry, &store, (&config, around.locations))?;
    let owner = principal::resolve();
    let show = ScopeGetInput { project };
    let now = registry.execute::<ScopeGet>(&owner, show)?;
    let mut notice = None;
    if args.changes() {
        let (languages, folders) = changed(args, &now);
        let input = ScopeSetInput {
            project,
            languages,
            folders,
        };
        let saved = registry.execute::<ScopeSet>(&owner, input)?;
        let key = if saved.changed {
            CLI_SCOPE_SAVED
        } else {
            CLI_SCOPE_UNCHANGED
        };
        notice = Some(say(around, key, &FluentArgs::new())?);
    }
    let answer = registry.execute::<ScopeGet>(&owner, show)?;
    if args.json {
        return serde_json::to_string_pretty(&answer)
            .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError));
    }
    let mut lines = lines_of(&answer, around)?;
    lines.extend(notice);
    Ok(lines.join(CLI_LINE_BREAK))
}
