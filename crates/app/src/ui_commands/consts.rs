//! Fixed values of the commands the browser UI uses.

use codeskimr_codes::i18n;

/// Id of the command that lists the projects.
pub const PROJECTS_LIST: &str = "projects.list";
/// Id of the command that registers a project and scans it.
pub const PROJECTS_ADD: &str = "projects.add";
/// Id of the command that scans a registered project again.
pub const PROJECTS_RESCAN: &str = "projects.rescan";
/// Id of the command that removes a project from the app.
pub const PROJECTS_REMOVE: &str = "projects.remove";
/// Id of the command that renames a project.
pub const PROJECTS_RENAME: &str = "projects.rename";
/// Title key of `projects.rename`.
pub const RENAME_TITLE: &str = i18n::COMMAND_PROJECTS_RENAME_TITLE;
/// Description key of `projects.rename`.
pub const RENAME_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_RENAME_DESCRIPTION;
/// Id of the command that tells the name a source would get.
pub const PROJECTS_SUGGEST_NAME: &str = "projects.suggest_name";
/// Title key of `projects.suggest_name`.
pub const SUGGEST_NAME_TITLE: &str = i18n::COMMAND_PROJECTS_SUGGEST_NAME_TITLE;
/// Description key of `projects.suggest_name`.
pub const SUGGEST_NAME_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_SUGGEST_NAME_DESCRIPTION;
/// Id of the command that tells whether a folder dialog exists on this machine.
pub const PROJECTS_FOLDER_DIALOG: &str = "projects.folder_dialog";
/// Id of the command that shows the folder dialog.
pub const PROJECTS_PICK_FOLDER: &str = "projects.pick_folder";

/// Id of the command that checks the tools of a project.
pub const PROJECTS_TOOLS: &str = "projects.tools";
/// Title key of `projects.tools`.
pub const TOOLS_TITLE: &str = i18n::COMMAND_PROJECTS_TOOLS_TITLE;
/// Description key of `projects.tools`.
pub const TOOLS_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_TOOLS_DESCRIPTION;
/// Id of the command that lists the indexers of this machine.
pub const TOOLS_INDEXERS: &str = "tools.indexers";
/// Title key of `tools.indexers`.
pub const INDEXERS_TITLE: &str = i18n::COMMAND_TOOLS_INDEXERS_TITLE;
/// Description key of `tools.indexers`.
pub const INDEXERS_DESCRIPTION: &str = i18n::COMMAND_TOOLS_INDEXERS_DESCRIPTION;
/// Id of the command that reports where level 2 stands for a project (ADR 0071).
pub const LEVEL2_STATUS: &str = "level2.status";
/// Title key of `level2.status`.
pub const LEVEL2_STATUS_TITLE: &str = i18n::COMMAND_LEVEL2_STATUS_TITLE;
/// Description key of `level2.status`.
pub const LEVEL2_STATUS_DESCRIPTION: &str = i18n::COMMAND_LEVEL2_STATUS_DESCRIPTION;
/// Id of the command that takes the level 2 confirmation of a project back (ADR 0071).
pub const LEVEL2_REVOKE: &str = "level2.revoke";
/// Title key of `level2.revoke`.
pub const LEVEL2_REVOKE_TITLE: &str = i18n::COMMAND_LEVEL2_REVOKE_TITLE;
/// Description key of `level2.revoke`.
pub const LEVEL2_REVOKE_DESCRIPTION: &str = i18n::COMMAND_LEVEL2_REVOKE_DESCRIPTION;
/// Id of the command that cancels a running scan.
pub const PROJECTS_CANCEL: &str = "projects.cancel";

/// Id of the command that returns the explorer tree.
pub const EXPLORER_OUTLINE: &str = "explorer.outline";
/// Id of the command that returns one symbol of the explorer.
pub const EXPLORER_SYMBOL: &str = "explorer.symbol";

/// Id of the command that returns the source of one symbol.
pub const EXPLORER_SOURCE: &str = "explorer.source";
/// Title key of `explorer.source`.
pub const SOURCE_TITLE: &str = i18n::COMMAND_EXPLORER_SOURCE_TITLE;
/// Description key of `explorer.source`.
pub const SOURCE_DESCRIPTION: &str = i18n::COMMAND_EXPLORER_SOURCE_DESCRIPTION;
/// Most lines of one declaration that `explorer.source` returns.
pub const SOURCE_MAX_LINES: u32 = 400;
/// Largest file that `explorer.source` reads, in bytes.
pub const SOURCE_MAX_BYTES: u64 = 4_194_304;
/// The most places in a declaration where a symbol is named that the source column gets.
pub const SOURCE_MAX_SPANS: usize = 4_000;

/// Id of the command that places the layers of the dependency map.
pub const GRAPH_LAYERS: &str = "graph.layers";
/// Title key of `graph.layers`.
pub const GRAPH_TITLE: &str = i18n::COMMAND_GRAPH_LAYERS_TITLE;
/// Description key of `graph.layers`.
pub const GRAPH_DESCRIPTION: &str = i18n::COMMAND_GRAPH_LAYERS_DESCRIPTION;

/// Id of the command of the quick search.
pub const SEARCH_QUERY: &str = "search.query";
/// Most hits the quick search returns.
pub const SEARCH_LIMIT: u32 = 50;
/// The fewest hits a project gets in a search over all projects.
pub const SEARCH_MIN_SHARE: u32 = 5;
/// The most files a search returns.
pub const SEARCH_FILE_LIMIT: u32 = 10;
/// Title key of `search.query`.
pub const SEARCH_TITLE: &str = i18n::COMMAND_SEARCH_QUERY_TITLE;
/// Description key of `search.query`.
pub const SEARCH_DESCRIPTION: &str = i18n::COMMAND_SEARCH_QUERY_DESCRIPTION;

/// Id of the command of the usages screen.
pub const USAGES_QUERY: &str = "usages.query";
/// Title key of `usages.query`.
pub const USAGES_TITLE: &str = i18n::COMMAND_USAGES_QUERY_TITLE;
/// Description key of `usages.query`.
pub const USAGES_DESCRIPTION: &str = i18n::COMMAND_USAGES_QUERY_DESCRIPTION;
/// Most references and symbols of one walk that `usages.query` returns.
pub const USAGES_LIMIT: u32 = 500;
/// Id of the command that returns the code lines of the hits of a usages query.
pub const USAGES_CODE: &str = "usages.code";
/// Title key of `usages.code`.
pub const USAGES_CODE_TITLE: &str = i18n::COMMAND_USAGES_CODE_TITLE;
/// Description key of `usages.code`.
pub const USAGES_CODE_DESCRIPTION: &str = i18n::COMMAND_USAGES_CODE_DESCRIPTION;
/// Most hits `usages.code` reads a line for; the hit list shows the first ones.
pub const USAGES_CODE_MAX_HITS: usize = 200;
/// Most lines `usages.code` reads from one file to reach all its hits; hits further down get no code.
pub const USAGES_CODE_MAX_SPAN: u32 = 5_000;
/// Most characters of one code line `usages.code` returns.
pub const USAGES_CODE_MAX_CHARS: usize = 240;
/// State of level 2 when no hit of the answer was verified by an indexer (ADR 0033): level 2 is off,
/// was not run, or did not reach the symbol.
pub const LEVEL2_UNAVAILABLE: &str = "unavailable";
/// State of level 2 when at least one hit of the answer was verified by an indexer.
pub const LEVEL2_ACTIVE: &str = "active";

/// Id of the command of the status bar.
pub const STATUS_INFO: &str = "status.info";
/// Title key of `status.info`.
pub const STATUS_TITLE: &str = i18n::COMMAND_STATUS_INFO_TITLE;
/// Description key of `status.info`.
pub const STATUS_DESCRIPTION: &str = i18n::COMMAND_STATUS_INFO_DESCRIPTION;
/// The version of the program, as `codeskimr --version` prints it.
pub const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Bytes in a megabyte, for the memory shown in the status bar.
pub const BYTES_PER_MEGABYTE: u64 = 1_048_576;

/// Shortcut of the new-project form.
pub const SHORTCUT_NEW: &str = "N";
/// Shortcut of the re-scan.
pub const SHORTCUT_RESCAN: &str = "R";

/// Title key of `projects.list`.
pub const LIST_TITLE: &str = i18n::COMMAND_PROJECTS_LIST_TITLE;
/// Description key of `projects.list`.
pub const LIST_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_LIST_DESCRIPTION;
/// Title key of `projects.add`.
pub const ADD_TITLE: &str = i18n::COMMAND_PROJECTS_ADD_TITLE;
/// Description key of `projects.add`.
pub const ADD_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_ADD_DESCRIPTION;
/// Title key of `projects.rescan`.
pub const RESCAN_TITLE: &str = i18n::COMMAND_PROJECTS_RESCAN_TITLE;
/// Description key of `projects.rescan`.
pub const RESCAN_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_RESCAN_DESCRIPTION;

/// Title key of `projects.remove`.
pub const REMOVE_TITLE: &str = i18n::COMMAND_PROJECTS_REMOVE_TITLE;
/// Description key of `projects.remove`.
pub const REMOVE_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_REMOVE_DESCRIPTION;

/// Title key of `projects.folder_dialog`.
pub const FOLDER_DIALOG_TITLE: &str = i18n::COMMAND_PROJECTS_FOLDER_DIALOG_TITLE;
/// Description key of `projects.folder_dialog`.
pub const FOLDER_DIALOG_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_FOLDER_DIALOG_DESCRIPTION;
/// Title key of `projects.pick_folder`.
pub const PICK_FOLDER_TITLE: &str = i18n::COMMAND_PROJECTS_PICK_FOLDER_TITLE;
/// Description key of `projects.pick_folder`.
pub const PICK_FOLDER_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_PICK_FOLDER_DESCRIPTION;

/// Title key of `projects.cancel`.
pub const CANCEL_TITLE: &str = i18n::COMMAND_PROJECTS_CANCEL_TITLE;
/// Description key of `projects.cancel`.
pub const CANCEL_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_CANCEL_DESCRIPTION;

/// How a project's source is named on the wire.
pub const KIND_FOLDER: &str = "folder";
/// How a Git source is named on the wire.
pub const KIND_GIT: &str = "git";
/// `database.state` of a project whose file is there.
pub const DATABASE_OK: &str = "ok";
/// `database.state` of a registered project whose file is missing.
pub const DATABASE_MISSING: &str = "missing";
/// `database.state` of a project that is being removed.
pub const DATABASE_REMOVING: &str = "removing";
/// The state of a project file that cannot be opened.
pub const DATABASE_UNREADABLE: &str = "unreadable";

/// Title key of `explorer.outline`.
pub const OUTLINE_TITLE: &str = i18n::COMMAND_EXPLORER_OUTLINE_TITLE;
/// Description key of `explorer.outline`.
pub const OUTLINE_DESCRIPTION: &str = i18n::COMMAND_EXPLORER_OUTLINE_DESCRIPTION;
/// Title key of `explorer.symbol`.
pub const SYMBOL_TITLE: &str = i18n::COMMAND_EXPLORER_SYMBOL_TITLE;
/// Description key of `explorer.symbol`.
pub const SYMBOL_DESCRIPTION: &str = i18n::COMMAND_EXPLORER_SYMBOL_DESCRIPTION;

/// Lines are stored from 0, the screen counts from 1.
pub const LINE_OFFSET: u32 = 1;
/// Longest chain of enclosing symbols in the breadcrumb.
pub const BREADCRUMB_DEPTH: usize = 8;

/// Rating label of a value within the good range.
pub const RATING_GOOD: &str = "good";
/// Rating label of a value within the medium range.
pub const RATING_MEDIUM: &str = "medium";
/// Rating label of a value beyond the medium range.
pub const RATING_POOR: &str = "poor";

/// Id of the command that exports a project as one HTML page.
pub const EXPORT_HTML: &str = "export.html";
/// Title key of `export.html`.
pub const EXPORT_TITLE: &str = i18n::COMMAND_EXPORT_HTML_TITLE;
/// Description key of `export.html`.
pub const EXPORT_DESCRIPTION: &str = i18n::COMMAND_EXPORT_HTML_DESCRIPTION;
/// Version of the data block inside the export, so the page can refuse data it does not know.
pub const EXPORT_FORMAT: u32 = 1;
/// Folder of the UI folder that holds the export page.
pub const EXPORT_BUNDLE_DIR: &str = "export";
/// The page the export is made from: the built UI in one file, with a place for the data.
pub const EXPORT_BUNDLE_FILE: &str = "export.html";
/// Separator of the paths in the list of embedded UI files.
pub const EXPORT_PATH_SEPARATOR: &str = "/";
/// The place in that file where the data block goes, exactly once.
pub const EXPORT_DATA_PLACEHOLDER: &str = "__CODESKIMR_EXPORT_DATA__";
/// A character that must not stand in the data block as it is: it could end the script element.
pub const EXPORT_ESCAPED_CHAR: char = '<';
/// What takes its place; JSON reads it back as the character.
pub const EXPORT_ESCAPE: &str = "\\u003c";
/// Steps of the walk that counts the direct usages of a symbol: one.
pub const DIRECT_DEPTH: u32 = 1;
/// Steps of the usages walk in the export: only the direct usages. Deeper walks grow with the
/// number of paths, not of symbols (440,000 of 450,000 hits of a 1,286 symbol project were deeper
/// than one step, 150 MB), so they stay with the app. The page offers no deeper select.
pub const EXPORT_DEPTH: u32 = 1;
/// Keys of the usages answer that the export packs: the two walks, their hits and the ends of a hit.
pub const EXPORT_INCOMING: &str = "incoming";
/// See [`EXPORT_INCOMING`].
pub const EXPORT_OUTGOING: &str = "outgoing";
/// See [`EXPORT_INCOMING`].
pub const EXPORT_HITS: &str = "hits";
/// See [`EXPORT_INCOMING`].
pub const EXPORT_FROM: &str = "from";
/// See [`EXPORT_INCOMING`].
pub const EXPORT_TO: &str = "to";
/// Key of the id in a place.
pub const EXPORT_ID: &str = "id";

/// The gap between two rows is divided by this to get the space a layer band adds above and
/// below its boxes.
pub const BAND_PAD_DIVISOR: u32 = 2;
/// A line of a wrapped row is centred: the free space is divided by this and put on the left.
pub const CENTRE_DIVISOR: u32 = 2;
/// The fewest boxes in a line of a row, whatever the configuration says.
pub const MIN_LINE_BOXES: u32 = 1;
/// Joins the namespaces in the breadcrumb of a type box.
pub const BREADCRUMB_JOIN: &str = " › ";
/// The margin of the picture applies on both sides.
pub const MARGIN_SIDES: u32 = 2;
/// Id of the command that lists the credentials of the Git projects.
pub const CREDENTIALS_LIST: &str = "credentials.list";
/// Id of the command that stores a credential.
pub const CREDENTIALS_SET: &str = "credentials.set";
/// Id of the command that removes a credential.
pub const CREDENTIALS_DELETE: &str = "credentials.delete";
/// Title key of `credentials.list`.
pub const CREDENTIALS_LIST_TITLE: &str = i18n::COMMAND_CREDENTIALS_LIST_TITLE;
/// Description key of `credentials.list`.
pub const CREDENTIALS_LIST_DESCRIPTION: &str = i18n::COMMAND_CREDENTIALS_LIST_DESCRIPTION;
/// Title key of `credentials.set`.
pub const CREDENTIALS_SET_TITLE: &str = i18n::COMMAND_CREDENTIALS_SET_TITLE;
/// Description key of `credentials.set`.
pub const CREDENTIALS_SET_DESCRIPTION: &str = i18n::COMMAND_CREDENTIALS_SET_DESCRIPTION;
/// Title key of `credentials.delete`.
pub const CREDENTIALS_DELETE_TITLE: &str = i18n::COMMAND_CREDENTIALS_DELETE_TITLE;
/// Description key of `credentials.delete`.
pub const CREDENTIALS_DELETE_DESCRIPTION: &str = i18n::COMMAND_CREDENTIALS_DELETE_DESCRIPTION;

/// Most types drawn on each side of the type in the middle of the map; the rest is counted.
pub const TYPE_SIDE_MAX: usize = 12;
/// The most namespaces drawn on each side of the namespace in the middle.
pub const NAMESPACE_SIDE_MAX: usize = 24;
/// Between the qualified name and the id in the name of a box, if two drawn types share a name.
pub const BOX_ID_SEPARATOR: &str = " #";
/// Row of the types that use the type in the middle.
pub const TYPE_ROW_USERS: u32 = 0;
/// Row of the type in the middle.
pub const TYPE_ROW_FOCUS: u32 = 1;
/// Row of the types the type in the middle uses.
pub const TYPE_ROW_USED: u32 = 2;

/// Id of the command that returns the preferences of the user (theme and language).
pub const PREFERENCES_GET: &str = "preferences.get";
/// Title key of `preferences.get`.
pub const PREFERENCES_GET_TITLE: &str = i18n::COMMAND_PREFERENCES_GET_TITLE;
/// Description key of `preferences.get`.
pub const PREFERENCES_GET_DESCRIPTION: &str = i18n::COMMAND_PREFERENCES_GET_DESCRIPTION;
/// Id of the command that stores the preferences of the user.
pub const PREFERENCES_SET: &str = "preferences.set";
/// Title key of `preferences.set`.
pub const PREFERENCES_SET_TITLE: &str = i18n::COMMAND_PREFERENCES_SET_TITLE;
/// Description key of `preferences.set`.
pub const PREFERENCES_SET_DESCRIPTION: &str = i18n::COMMAND_PREFERENCES_SET_DESCRIPTION;
/// File in the data folder that holds the preferences.
pub const PREFERENCES_FILE: &str = "preferences.json";
/// Id of the command that returns the settings the UI saved (ADR 0103).
pub const UI_SETTINGS_GET: &str = "ui_settings.get";
/// Title key of `ui_settings.get`.
pub const UI_SETTINGS_GET_TITLE: &str = i18n::COMMAND_UI_SETTINGS_GET_TITLE;
/// Description key of `ui_settings.get`.
pub const UI_SETTINGS_GET_DESCRIPTION: &str = i18n::COMMAND_UI_SETTINGS_GET_DESCRIPTION;
/// Id of the command that saves or forgets the memory limit of one indexer.
pub const UI_SETTINGS_SET_INDEXER_MEMORY: &str = "ui_settings.set_indexer_memory";
/// Title key of `ui_settings.set_indexer_memory`.
pub const UI_SETTINGS_SET_INDEXER_MEMORY_TITLE: &str =
    i18n::COMMAND_UI_SETTINGS_SET_INDEXER_MEMORY_TITLE;
/// Description key of `ui_settings.set_indexer_memory`.
pub const UI_SETTINGS_SET_INDEXER_MEMORY_DESCRIPTION: &str =
    i18n::COMMAND_UI_SETTINGS_SET_INDEXER_MEMORY_DESCRIPTION;
/// Ending of the file a content is written to before it replaces the real one.
pub const TEMP_SUFFIX: &str = ".tmp";
/// Longest name of a theme or a language that is stored.
pub const PREFERENCE_MAX_CHARS: usize = 32;
/// Characters besides lower-case letters and digits that a stored name may contain.
pub const PREFERENCE_EXTRA_CHAR: char = '-';
/// Largest preferences file that is read; anything bigger is ignored.
pub const PREFERENCES_MAX_BYTES: u64 = 4096;
/// Mode of the preferences file: the owner may read and write, nobody else.
#[cfg(unix)]
pub const PREFERENCES_FILE_MODE: u32 = 0o600;
/// Separator in the name of that file.
pub const TEMP_SEPARATOR: &str = ".";

/// Id of the command that shows the layers of a project and the namespaces to build them from.
pub const LAYERS_GET: &str = "layers.get";
/// Title key of `layers.get`.
pub const LAYERS_GET_TITLE: &str = i18n::COMMAND_LAYERS_GET_TITLE;
/// Description key of `layers.get`.
pub const LAYERS_GET_DESCRIPTION: &str = i18n::COMMAND_LAYERS_GET_DESCRIPTION;
/// Id of the command that keeps a symbol on purpose (an entry of `[[unused.ignore]]`).
pub const UNUSED_KEEP: &str = "unused.keep";
/// Title key of `unused.keep`.
pub const UNUSED_KEEP_TITLE: &str = i18n::COMMAND_UNUSED_KEEP_TITLE;
/// Description key of `unused.keep`.
pub const UNUSED_KEEP_DESCRIPTION: &str = i18n::COMMAND_UNUSED_KEEP_DESCRIPTION;
/// Id of the command that takes an entry out of `[[unused.ignore]]` again.
pub const UNUSED_LIFT: &str = "unused.lift";
/// Title key of `unused.lift`.
pub const UNUSED_LIFT_TITLE: &str = i18n::COMMAND_UNUSED_LIFT_TITLE;
/// Description key of `unused.lift`.
pub const UNUSED_LIFT_DESCRIPTION: &str = i18n::COMMAND_UNUSED_LIFT_DESCRIPTION;
/// Id of the command that writes the layers into the project file.
pub const LAYERS_SET: &str = "layers.set";
/// Title key of `layers.set`.
pub const LAYERS_SET_TITLE: &str = i18n::COMMAND_LAYERS_SET_TITLE;
/// Description key of `layers.set`.
pub const LAYERS_SET_DESCRIPTION: &str = i18n::COMMAND_LAYERS_SET_DESCRIPTION;

/// Id of the command that lists annotations.
pub const ANNOTATIONS_LIST: &str = "annotations.list";
/// Id of the command that counts annotations per symbol.
pub const ANNOTATIONS_SUMMARY: &str = "annotations.summary";
/// Id of the command that adds an annotation of any kind.
pub const ANNOTATIONS_CREATE: &str = "annotations.create";
/// Id of the command that marks a symbol as critical.
pub const ANNOTATIONS_MARK_CRITICAL: &str = "annotations.mark_critical";
/// Id of the command that adds a note.
pub const ANNOTATIONS_ADD_NOTE: &str = "annotations.add_note";
/// Id of the command that adds a task.
pub const ANNOTATIONS_ADD_TASK: &str = "annotations.add_task";
/// Id of the command that changes an annotation.
pub const ANNOTATIONS_UPDATE: &str = "annotations.update";
/// Id of the command that deletes an annotation.
pub const ANNOTATIONS_DELETE: &str = "annotations.delete";
/// Id of the command that lists alerts.
pub const ALERTS_LIST: &str = "alerts.list";
/// Id of the command that dismisses one alert.
pub const ALERTS_DISMISS: &str = "alerts.dismiss";
/// Id of the command that dismisses the alerts of a filter.
pub const ALERTS_DISMISS_ALL: &str = "alerts.dismiss_all";
/// Default shortcut of `annotations.mark_critical` (blueprint 15.4).
pub const SHORTCUT_CRITICAL: &str = "K";
/// Default shortcut of `annotations.add_note`.
pub const SHORTCUT_NOTE: &str = "N";
/// Default shortcut of `annotations.add_task`.
pub const SHORTCUT_TASK: &str = "T";
/// The author of every annotation until the team edition has accounts (ADR 0068).
pub const LOCAL_AUTHOR: &str = "local";

/// `Level2Answer::state` when a background task runs level 2.
pub const LEVEL2_STARTED: &str = "started";
/// `Level2Answer::state` when level 2 did not start; the reason says why.
pub const LEVEL2_NOT_STARTED: &str = "not_started";
/// `Level2Answer::state` when the caller asked for level 1 only.
pub const LEVEL2_NOT_REQUESTED: &str = "not_requested";

/// Id of the command that lists the last scans of a project (ADR 0097).
pub const SCANS_LIST: &str = "scans.list";
/// Title key of `scans.list`.
pub const SCANS_LIST_TITLE: &str = i18n::COMMAND_SCANS_LIST_TITLE;
/// Description key of `scans.list`.
pub const SCANS_LIST_DESCRIPTION: &str = i18n::COMMAND_SCANS_LIST_DESCRIPTION;
/// Id of the command that summarises a project for the statistics screen (ADR 0081).
pub const STATS_OVERVIEW: &str = "stats.overview";
/// Title key of `stats.overview`.
pub const STATS_OVERVIEW_TITLE: &str = i18n::COMMAND_STATS_OVERVIEW_TITLE;
/// Description key of `stats.overview`.
pub const STATS_OVERVIEW_DESCRIPTION: &str = i18n::COMMAND_STATS_OVERVIEW_DESCRIPTION;
/// Id of the command that ranks the symbols of a project (ADR 0081).
pub const STATS_RANKING: &str = "stats.ranking";
/// Title key of `stats.ranking`.
pub const STATS_RANKING_TITLE: &str = i18n::COMMAND_STATS_RANKING_TITLE;
/// Description key of `stats.ranking`.
pub const STATS_RANKING_DESCRIPTION: &str = i18n::COMMAND_STATS_RANKING_DESCRIPTION;
/// Rows of a ranking if the caller names no limit.
pub const STATS_LIMIT: u32 = 100;
/// Most rows of a ranking, whatever the caller asks for.
pub const STATS_LIMIT_MAX: u32 = 500;
/// Rows of each card of the overview.
pub const STATS_TOP: u32 = 5;

/// The separator between the namespace and the name in the qualified name of a type that the tree
/// leaves out of its answer (the reader puts it together again).
pub(super) const QUALIFIED_SEPARATOR: &str = ".";

/// Id of the command that shows the settings in effect with their origin.
pub const SETTINGS_EFFECTIVE: &str = "settings.effective";
/// Title key of `settings.effective`.
pub const SETTINGS_EFFECTIVE_TITLE: &str = i18n::COMMAND_SETTINGS_EFFECTIVE_TITLE;
/// Description key of `settings.effective`.
pub const SETTINGS_EFFECTIVE_DESCRIPTION: &str = i18n::COMMAND_SETTINGS_EFFECTIVE_DESCRIPTION;
/// Origin name of a value from `config/default.toml`.
pub const ORIGIN_DEFAULT: &str = "default";
/// Origin name of a value from the user configuration file.
pub const ORIGIN_USER: &str = "user";
/// Origin name of a value from the `.codeskimr.toml` of a project.
pub const ORIGIN_PROJECT: &str = "project";
/// Origin of a value the UI saved (ADR 0103).
pub const ORIGIN_UI: &str = "ui";
/// Origin name of a value from a command line flag (`--set`).
pub const ORIGIN_CLI: &str = "cli";
/// Joins key and value in the text of a command line override.
pub const OVERRIDE_JOIN: char = '=';

/// Id of the command that shows what a project covers and what it leaves out.
pub const SCOPE_GET: &str = "scope.get";
/// Title key of `scope.get`.
pub const SCOPE_GET_TITLE: &str = i18n::COMMAND_SCOPE_GET_TITLE;
/// Description key of `scope.get`.
pub const SCOPE_GET_DESCRIPTION: &str = i18n::COMMAND_SCOPE_GET_DESCRIPTION;
/// Id of the command that stores what a project leaves out.
pub const SCOPE_SET: &str = "scope.set";
/// Title key of `scope.set`.
pub const SCOPE_SET_TITLE: &str = i18n::COMMAND_SCOPE_SET_TITLE;
/// Description key of `scope.set`.
pub const SCOPE_SET_DESCRIPTION: &str = i18n::COMMAND_SCOPE_SET_DESCRIPTION;
