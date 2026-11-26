# 0099 – Agent package: `get_context` and the query commands of the command line

Status: accepted (the scope of release 1.1 "Change and agent package" was decided by the product owner on 2026-11-23; the mocks of 1.1 on 2026-11-26)

## Context

The concept for the time after 1.0 (`entscheidungen/erweiterungen-nach-1-0.md`, items K1 and K3) asks for two things for code agents and scripts. First, an agent that wants to change a symbol calls `get_symbol`, `find_usages`, `get_dependencies`, `impact_of_change` and `list_annotations` one after the other and pays for five answers. Second, scripts, CI and agents without MCP have no way to ask the questions of the MCP tools: the command line has `--json` for `scan`, `check`, `annotations` and `alerts`, but no query commands.

## Options

1. A new MCP tool `get_context` that gathers the five answers, and query commands that call the tools of the MCP crate.
2. Query commands as registered commands of the registry, and a new tool next to them.
3. Leave it to the agents.

## Decision

Option 1.

- **`get_context`** is a read-only tool (`read`, no source text). Its answer holds the symbol with signature, documentation, layer and origin, its direct callers (verified first, tests left out), what it depends on, the distinct files of the symbols a change can reach, and its annotations. Every list is `{ total, items }`. The argument `budget` (`short`, `medium`, `long`) says how many entries a list holds, from the settings `mcp.context_items_short`, `mcp.context_items_medium` and `mcp.context_items_long` (3, 10, 50), so the agent says how much room it has and CodeSkimr cuts, instead of the agent reading whole files. A cut list is announced with the notice `MCP-501`. `preliminary` is `true` while a listed caller or affected symbol is unchecked, as for `find_usages`. The tool list grows from 13 to 14 tools; the concept asks to stay at 17 at most.
- **Query commands.** `codeskimr symbols`, `usages`, `impact` and `context` run the tools `search_symbols`, `find_usages`, `impact_of_change` and `get_context` of the MCP crate over the same store, with the same limits and the same answers, so there is one logic and no second implementation to keep equal. They are not commands of the registry: those serve the UI and REST with their own shapes, and the answers of the tools are the ones agents already know.
- **Promises of the format** (concept K3): `--json` prints the answer of the tool plus `schema` (version of the format, `1`) and `stale`; a failure is JSON with the registered code on standard output and exit code `1`; nothing else goes to standard output.
- **No hidden scan.** The commands read the newest scan. `stale` is `true` if a file of a folder project changed, was added or went away since that scan (the files are hashed like a scan hashes them, without parsing), `false` if not, and `null` for a Git source, where only a scan updates the copy. The text output ends with a line that says so.
- **ADR 0092.** The counterpart of the text output is the explorer with its symbol card, the search and the usages screen; `--json` is an exception of that ADR whose counterpart is the REST interface. `get_context` has no screen of its own: the UI shows its parts in the places where people look for them.
- Not part of this decision: `unused`, the comparison of scans and the change view (own packages of 1.1).

## Consequences

- Blueprint 10.2 and 10.4 name the commands and the tool.
- `McpLimits` has three more settings; the validation keeps them in order and below `mcp.max_limit`.
- A change of the answer of a tool changes the JSON of a command: the version `schema` grows then.
- The commands load the language packs to wire the tools (as `codeskimr mcp` does), and a check of `stale` walks the project folder, so on a very large project the first answer takes seconds, not milliseconds.
