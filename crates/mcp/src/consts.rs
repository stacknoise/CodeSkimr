//! Fixed values of the MCP server.

/// Name of the tool that searches symbols by name.
pub const TOOL_SEARCH_SYMBOLS: &str = "search_symbols";
/// Name of the tool that shows one symbol.
pub const TOOL_GET_SYMBOL: &str = "get_symbol";
/// Name of the tool that shows the architecture of a project.
pub const TOOL_GET_ARCHITECTURE: &str = "get_architecture";

/// Name the server reports in the `initialize` answer.
pub const SERVER_NAME: &str = "codeskimr";
/// Language of all MCP texts: they are written for code agents.
pub const TEXT_LANGUAGE: &str = "en";

/// Smallest number of results a caller may ask for.
pub const MIN_LIMIT: u32 = 1;
/// Lines in domain spans are zero-based, in the answers one-based.
pub const LINE_OFFSET: u32 = 1;

/// Parameter of an error: the project id.
pub const PARAM_PROJECT: &str = "project";
/// Parameter of an error: the symbol the caller named.
pub const PARAM_SYMBOL: &str = "symbol";
/// Parameter of an error: what is wrong with an argument.
pub const PARAM_DETAIL: &str = "detail";
/// Parameter of a notice: the limit that was applied.
pub const PARAM_LIMIT: &str = "limit";
/// Parameter of a notice: how many results there were.
pub const PARAM_TOTAL: &str = "total";

/// Argument name reported when its value is invalid: the search text.
pub const ARG_QUERY: &str = "query";
/// Argument name reported when its value is invalid: the symbol kind.
pub const ARG_KIND: &str = "kind";
/// Argument name reported when its value is invalid: the symbol.
pub const ARG_SYMBOL: &str = "symbol";

/// Version the server reports in the `initialize` answer.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Between the code and its description in the text of an error.
pub const ID_SEPARATOR: &str = ": ";
