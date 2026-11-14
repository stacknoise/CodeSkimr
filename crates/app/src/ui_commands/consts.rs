//! Fixed values of the commands the browser UI uses.

use codeskimr_codes::i18n;

/// Id of the command that lists the projects.
pub const PROJECTS_LIST: &str = "projects.list";
/// Id of the command that registers a project and scans it.
pub const PROJECTS_ADD: &str = "projects.add";
/// Id of the command that scans a registered project again.
pub const PROJECTS_RESCAN: &str = "projects.rescan";

/// Id of the command that checks the tools of a project.
pub const PROJECTS_TOOLS: &str = "projects.tools";
/// Title key of `projects.tools`.
pub const TOOLS_TITLE: &str = i18n::COMMAND_PROJECTS_TOOLS_TITLE;
/// Description key of `projects.tools`.
pub const TOOLS_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_TOOLS_DESCRIPTION;

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
/// State of level 2 while no indexer is wired (ADR 0033).
pub const LEVEL2_UNAVAILABLE: &str = "unavailable";

/// Id of the command of the status bar.
pub const STATUS_INFO: &str = "status.info";
/// Title key of `status.info`.
pub const STATUS_TITLE: &str = i18n::COMMAND_STATUS_INFO_TITLE;
/// Description key of `status.info`.
pub const STATUS_DESCRIPTION: &str = i18n::COMMAND_STATUS_INFO_DESCRIPTION;
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

/// How a project's source is named on the wire.
pub const KIND_FOLDER: &str = "folder";
/// How a Git source is named on the wire.
pub const KIND_GIT: &str = "git";

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
/// The place in that file where the data block goes, exactly once.
pub const EXPORT_DATA_PLACEHOLDER: &str = "__CODESKIMR_EXPORT_DATA__";
/// A character that must not stand in the data block as it is: it could end the script element.
pub const EXPORT_ESCAPED_CHAR: char = '<';
/// What takes its place; JSON reads it back as the character.
pub const EXPORT_ESCAPE: &str = "\\u003c";
/// Steps of the usages walk in the export: as deep as the configuration allows.
pub const EXPORT_DEPTH: u32 = 0;
/// Steps of the walk that counts the direct usages.
pub const EXPORT_DIRECT_DEPTH: u32 = 1;

/// The gap between two rows is divided by this to get the space a layer band adds above and
/// below its boxes.
pub const BAND_PAD_DIVISOR: u32 = 2;
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
