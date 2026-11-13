# MCP server (WP-26, WP-36, WP-37)

Implements the first part of blueprint section 10.4: the stdio server with the three tools of milestone M1 and, since WP-36, the query tools `find_usages` and `impact_of_change` and, since WP-37, the check tools `find_similar_code`, `find_constants` and `check_conventions`. The remaining tools (`get_dependencies`, `rescan`), the resources and the HTTP transport follow in M2. `codeskimr mcp` is wired up in WP-25 (see `docs/cli.md`).

## Structure

- `codeskimr_mcp::Tools` holds the logic, synchronous, behind the ports `ProjectRepository`, `CodeIndex` and `Clock`. It is tested without any protocol.
- `codeskimr_mcp::McpServer` implements `rmcp::ServerHandler`: tool list, argument parsing, error mapping. Tool calls run on a blocking thread, so a slow query never stalls the protocol loop.
- `codeskimr_mcp::serve_stdio(server)` serves one client over stdin/stdout. The composition root builds `Tools` (store, clock, `config.mcp`, default project) and `McpServer::new(tools, &Messages::load()?)`.
- The ports are handed over as one `Ports` value (projects, index, usages, similarity, snippets, checker, clock), so a tool that needs another port adds a field instead of a constructor argument. The port `CodeIndex` (`codeskimr_application::ports`) offers `latest_snapshot`, `search`, `symbol` and `layer_edges`. `codeskimr_storage::SqliteStore` implements it. The crate rule stays: `mcp` never depends on `secrets` or on an adapter.

## Tools

All of them are read-only (`readOnlyHint`). Every answer is structured JSON and states the snapshot it is based on (`snapshot.id`, `snapshot.age_seconds`). `project` is optional everywhere and defaults to the project the server was started for.

| Tool | Arguments | Answer |
|---|---|---|
| `search_symbols` | `query`, `kind?`, `project?`, `limit?` | `hits`: id, name, qualified name, kind, path, best match first |
| `get_symbol` | `symbol` (id or qualified name), `project?` | signature, doc, visibility, lines (one-based), layer, metrics, members |
| `find_usages` | `symbol`, `direction?`, `depth?`, `include_tests?`, `project?`, `limit?` | `usages` (nearest step first): `depth`, `kind`, `confidence`, `path`, `line` and `column` (one-based), `from` and `to` symbol; `total`, `direction`, `depth` as applied |
| `impact_of_change` | `symbol`, `project?`, `limit?` | `symbols` a change can reach: `depth`, `layer`, `visibility`, `public_interface`; `layers` with counts; `public_interfaces`; `total` |
| `find_similar_code` | `symbol` or `snippet` + `language`, `min_similarity_percent?`, `project?`, `limit?` | `matches` (most similar first): id, qualified name, kind, `path`, `line`, `end_line` (one-based), `similarity_percent`; `min_similarity_percent` as applied |
| `find_constants` | `query`, `project?`, `limit?` | `constants`: id, name, qualified name, `path`, `signature` (the declaration as written, `null` if the pack has none) |
| `check_conventions` | `paths?`, `project?`, `limit?` | `findings` with `CHK` codes, `total`, `files_checked`, `unchecked`, `unmatched`, `not_checked` (see `docs/conventions.md`) |
| `get_architecture` | `project?` | `layers` with their namespaces, `dependencies` between layers with count, `violates_layering`, `in_cycle` |

## Usages and impact

Both tools sit on the `UsageIndex` port (`docs/usages.md`) and share the rules of that walk.

- `symbol` is an id or a qualified name. An unknown symbol is `MCP-202`, never an empty list: "no usages" and "no such symbol" are different answers.
- `find_usages`: `direction` is `incoming` (default: who uses the symbol) or `outgoing` (what it uses); anything else is `MCP-101` with `params.detail = direction`. `depth` is at least 1 (zero is `MCP-101`), defaults to `mcp.default_depth` and is held to `mcp.max_depth` (1 and 5). `include_tests` is off by default.
- `confidence` is `certain` or `probable` on every usage. A probable usage is the resolver's best guess (blueprint 9.4); an agent should verify it before relying on it.
- Usages end at the symbol that contains the occurrence: the usages of a type do not include those of its members.
- `impact_of_change` always walks `mcp.max_depth` steps and always counts references from tests. It lists the symbols reached, nearest first, with their layer (by namespace, as in `get_symbol`) and `public_interface` (visibility `public`). `layers` and `public_interfaces` count the symbols that are listed, so with a cut result they cover the first `limit` symbols; `total` is the full number.

## Similar code and constants

- `find_similar_code` answers "does this already exist?". The subject is either a stored function (`symbol`, an id or qualified name) or a `snippet` with the id of its language pack (`language`, for example `csharp`); giving both or neither is `MCP-101` with `params.detail = symbol or snippet`. The snippet is parsed like the content of a file and fingerprinted with the same normalisation and parameters as a scan (identifiers and literals do not matter), so a renamed copy of a function is found. Give one complete function that the pack parses at the top of a file; a method on its own in a language that needs a class around it gets different syntax nodes and matches less. A snippet larger than `mcp.max_snippet_bytes`, one too short to have fingerprints (`similarity.min_tokens`) and a symbol without stored fingerprints (not a function, or too short) are `MCP-101` with `params.detail` naming `snippet` or `symbol`; an unknown or missing language pack is `PRS-201`.
- The candidates are the functions of the newest snapshot, compared by the Jaccard similarity of their fingerprints, the measure of the duplicate check (`docs/storage.md`). `min_similarity_percent` (1 to 100, default `mcp.default_similarity_percent` = 50) is held outside that range as `MCP-101`. A stored function never lists itself. A snapshot taken before the fingerprints were stored (schema version 3) has none; scan again.
- `find_constants` is a name search restricted to the symbol kind `constant`, with the declaration of each hit. Config keys and translation keys, which blueprint 10.4 also names, are not covered yet (issue #100).
- `check_conventions` is described in `docs/conventions.md`. It reads the files of the project afresh and needs the permissions `read` and `read_source`; it does not use the snapshot, so its answer carries no `snapshot`.

## Limits

`limit` defaults to `mcp.default_limit` and is held to `1..=mcp.max_limit` (`config/default.toml`, 20 and 100). When a request is cut to the maximum (limit or, for `find_usages`, depth, which then appears in `notice.params.depth`), when `find_usages` or `impact_of_change` has more results than `limit`, when `search_symbols` has more hits than `limit`, or when `get_symbol` lists more members than the maximum, the answer carries a `notice` with code `MCP-501` and the applied `limit` (and `total` for members). A `kind` filter is applied after the search over the widest allowed result, so it can return fewer hits than `limit` even if more exist.

## Errors

A failing call is a normal tool result with `isError: true` and, in `structuredContent` and as text, `{ "code", "description", "params" }` (blueprint 11.5). The description is English. Codes:

| Code | When |
|---|---|
| `MCP-101` | arguments invalid or unknown tool; `params.detail` names the argument |
| `MCP-201` | the project is not registered; `params.project` |
| `MCP-202` | the symbol does not exist; `params.symbol` |
| `MCP-203` | the project has no snapshot yet |
| `MCP-501` | notice, not an error: the result was cut to the limit |
| others | the code of a failing port (for example `STO-302`), unchanged |

## Texts

Tool descriptions and the server instructions are written for code agents and live in `locales/{en,de}/mcp.ftl`. The German file carries the English text on purpose: agents get English in every language.

## Known limits

- `get_architecture` returns the layers and the measured dependencies. The allowed directions and the conventions are not part of the project model yet (see `docs/storage.md`, `violates_layering`); they join the answer with the layer rule of `check_conventions` (issue #45). The use case behind `check_conventions` and the tool exist (`docs/conventions.md`, `codeskimr check`); the layer rule joins it with issue #45.
- Paths from arguments (blueprint S-05): `check_conventions` takes paths, but only to select among the files of the project's own file list; a path is never opened, so `..` or an absolute path selects nothing. `rescan` will have to canonicalise against the registered roots.
- A qualified name shared by several symbols (overloads) answers with the lowest symbol id; use the id from `search_symbols` to be exact.
