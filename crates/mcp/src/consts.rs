//! Fixed values of the MCP server.

use codeskimr_domain::Permission;

/// What every tool of this server needs: all of them only read structure, metrics and symbol
/// signatures. A tool that returns source text would need `Permission::ReadSource` instead.
pub const READ_TOOL_PERMISSION: Permission = Permission::Read;

/// Name of the tool that searches symbols by name.
pub const TOOL_SEARCH_SYMBOLS: &str = "search_symbols";
/// Name of the tool that shows one symbol.
pub const TOOL_GET_SYMBOL: &str = "get_symbol";
/// Name of the tool that shows the architecture of a project.
pub const TOOL_GET_ARCHITECTURE: &str = "get_architecture";
/// Name of the tool that lists the usages of a symbol.
pub const TOOL_FIND_USAGES: &str = "find_usages";
/// Name of the tool that shows what a change of a symbol can reach.
pub const TOOL_IMPACT_OF_CHANGE: &str = "impact_of_change";
/// Name of the tool that looks for functions that resemble a piece of code.
pub const TOOL_FIND_SIMILAR_CODE: &str = "find_similar_code";
/// Name of the tool that looks for existing constants.
pub const TOOL_FIND_CONSTANTS: &str = "find_constants";
/// Name of the tool that checks code against the rules of blueprint section 3.
pub const TOOL_CHECK_CONVENTIONS: &str = "check_conventions";

/// Value of `direction`: the walk follows the users of the symbol.
pub const DIRECTION_INCOMING: &str = "incoming";
/// Value of `direction`: the walk follows what the symbol uses.
pub const DIRECTION_OUTGOING: &str = "outgoing";

/// Name the server reports in the `initialize` answer.
pub const SERVER_NAME: &str = "codeskimr";
/// Language of all MCP texts: they are written for code agents.
pub const TEXT_LANGUAGE: &str = "en";

/// Smallest number of results a caller may ask for.
pub const MIN_LIMIT: u32 = 1;
/// Lines in domain spans are zero-based, in the answers one-based.
pub const LINE_OFFSET: u32 = 1;
/// Columns in domain spans are zero-based, in the answers one-based.
pub const COLUMN_OFFSET: u32 = 1;

/// Parameter of an error: the project id.
pub const PARAM_PROJECT: &str = "project";
/// Parameter of an error: the symbol the caller named.
pub const PARAM_SYMBOL: &str = "symbol";
/// Parameter of an error: what is wrong with an argument.
pub const PARAM_DETAIL: &str = "detail";
/// Parameter of a notice: the limit that was applied.
pub const PARAM_LIMIT: &str = "limit";
/// Parameter of a notice: the depth that was applied.
pub const PARAM_DEPTH: &str = "depth";
/// Parameter of a notice: how many results there were.
pub const PARAM_TOTAL: &str = "total";

/// Argument name reported when its value is invalid: the search text.
pub const ARG_QUERY: &str = "query";
/// Argument name reported when its value is invalid: the symbol kind.
pub const ARG_KIND: &str = "kind";
/// Argument name reported when its value is invalid: the symbol.
pub const ARG_SYMBOL: &str = "symbol";
/// Argument name reported when its value is invalid: the direction of a walk.
pub const ARG_DIRECTION: &str = "direction";
/// Argument name reported when its value is invalid: the depth of a walk.
pub const ARG_DEPTH: &str = "depth";

/// Argument name reported when its value is invalid: the code snippet.
pub const ARG_SNIPPET: &str = "snippet";
/// Argument name reported when its value is invalid: the language of the snippet.
pub const ARG_LANGUAGE: &str = "language";
/// Argument name reported when its value is invalid: the minimum similarity.
pub const ARG_MIN_SIMILARITY: &str = "min_similarity_percent";
/// Argument name reported when `symbol` and `snippet` are both given or both missing.
pub const ARG_SYMBOL_OR_SNIPPET: &str = "symbol or snippet";
/// Largest similarity in percent.
pub const PERCENT_MAX: u32 = 100;

/// Version the server reports in the `initialize` answer.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Between the code and its description in the text of an error.
pub const ID_SEPARATOR: &str = ": ";
