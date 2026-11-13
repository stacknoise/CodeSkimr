# MCP server (WP-26, WP-36, WP-37, issue #109)

Implements the first part of blueprint section 10.4: the stdio server with the three tools of milestone M1 and, since WP-36, the query tools `find_usages` and `impact_of_change` and, since WP-37, the check tools `find_similar_code`, `find_constants` and `check_conventions`. Issue #109 adds `get_dependencies` and `rescan`, which completes the tool list of milestone M2. The resources follow. WP-48 added the HTTP transport (below). `codeskimr mcp` is wired up in WP-25 (see `docs/cli.md`).

## Structure

- `codeskimr_mcp::Tools` holds the logic, synchronous, behind the ports `ProjectRepository`, `CodeIndex` and `Clock`. It is tested without any protocol.
- `codeskimr_mcp::McpServer` implements `rmcp::ServerHandler`: tool list, argument parsing, error mapping. Tool calls run on a blocking thread, so a slow query never stalls the protocol loop.
- `codeskimr_mcp::serve_stdio(server)` serves one client over stdin/stdout. The composition root builds `Tools` (store, clock, `config.mcp`, default project) and `McpServer::new(tools, &Messages::load()?)`.
- The ports are handed over as one `Ports` value (projects, index, usages, similarity, snippets, checker, clock, scanner), so a tool that needs another port adds a field instead of a constructor argument. The port `CodeIndex` (`codeskimr_application::ports`) offers `latest_snapshot`, `search`, `symbol` and `layer_edges`. `codeskimr_storage::SqliteStore` implements it. The crate rule stays: `mcp` never depends on `secrets` or on an adapter.

## Transports

- **stdio:** `codeskimr mcp`, one project (default project of the calls), the local owner.
- **Streamable HTTP:** `POST /mcp` of the running app (`codeskimr serve`), see `docs/server.md`. `codeskimr_mcp::http_service(&server)` returns the `tower` service; it is stateless and answers with JSON. Tools are built with `Tools::unscoped`, so a call without `project` is `MCP-101` (`params.detail = project`). The token and the `Host` and `Origin` checks are the job of the web server, not of this crate.

## Tools

All of them are read-only (`readOnlyHint`) except `rescan`, which is announced with `readOnlyHint: false`. Every answer is structured JSON and states the snapshot it is based on (`snapshot.id`, `snapshot.age_seconds`). `project` is optional everywhere and defaults to the project the server was started for.

| Tool | Arguments | Answer |
|---|---|---|
| `search_symbols` | `query`, `kind?`, `project?`, `limit?` | `hits`: id, name, qualified name, kind, path, best match first |
| `get_symbol` | `symbol` (id or qualified name), `project?` | signature, doc, visibility, lines (one-based), layer, metrics, members |
| `find_usages` | `symbol`, `direction?`, `depth?`, `include_tests?`, `project?`, `limit?` | `usages` (nearest step first): `depth`, `kind`, `confidence`, `verified`, `path`, `line` and `column` (one-based), `from` and `to` symbol; `total`, `verified` and `unchecked` (counts of the listed usages), `preliminary`, `direction`, `depth` as applied |
| `impact_of_change` | `symbol`, `project?`, `limit?` | `symbols` a change can reach: `depth`, `layer`, `visibility`, `public_interface`, `verified`; `layers` with counts; `public_interfaces`; `verified` and `unchecked` (counts of the listed symbols), `preliminary`; `total` |
| `get_dependencies` | `symbol` or `layer`, `direction?`, `include_tests?`, `project?`, `limit?` | `target`, `direction`, `dependencies` (most references first): id, qualified name, kind, `path`, `layer`, `count`, `violates_layering`; `layers` with count, `violates_layering`, `in_cycle`; `violations`, `total` |
| `find_similar_code` | `symbol` or `snippet` + `language`, `min_similarity_percent?`, `project?`, `limit?` | `matches` (most similar first): id, qualified name, kind, `path`, `line`, `end_line` (one-based), `similarity_percent`; `min_similarity_percent` as applied |
| `find_constants` | `query`, `project?`, `limit?` | `constants`: id, name, qualified name, `path`, `signature` (the declaration as written, `null` if the pack has none) |
| `check_conventions` | `paths?`, `project?`, `limit?` | `findings` with `CHK` codes, `total`, `files_checked`, `unchecked`, `unmatched`, `not_checked` (see `docs/conventions.md`) |
| `get_architecture` | `project?` | `layers` with their namespaces, `dependencies` between layers with count, `violates_layering`, `in_cycle` |
| `rescan` | `project?` | `snapshot` (the new one), `mode` (`incremental`), `files`, `symbols`, `references`, `duration_ms` |

## Usages and impact

Both tools sit on the `UsageIndex` port (`docs/usages.md`) and share the rules of that walk.

- `symbol` is an id or a qualified name. An unknown symbol is `MCP-202`, never an empty list: "no usages" and "no such symbol" are different answers.
- `find_usages`: `direction` is `incoming` (default: who uses the symbol) or `outgoing` (what it uses); anything else is `MCP-101` with `params.detail = direction`. `depth` is at least 1 (zero is `MCP-101`), defaults to `mcp.default_depth` and is held to `mcp.max_depth` (1 and 5). `include_tests` is off by default.
- `confidence` is `certain` or `probable` on every usage. A probable usage is the resolver's best guess (blueprint 9.4); an agent should verify it before relying on it.
- `verified` is `true` only if level 2 produced the usage (ADR 0033); until level 2 exists every usage is `false`. Verified usages come first; `verified` and `unchecked` count the listed ones apart, and `preliminary` is `true` while any listed one is unchecked. An agent must not treat a preliminary answer as complete or exact. `impact_of_change` marks a symbol `verified` only if a path of verified references leads to it.
- Usages end at the symbol that contains the occurrence: the usages of a type do not include those of its members.
- `impact_of_change` always walks `mcp.max_depth` steps and always counts references from tests. It lists the symbols reached, nearest first, with their layer (by namespace, as in `get_symbol`) and `public_interface` (visibility `public`). `layers` and `public_interfaces` count the symbols that are listed, so with a cut result they cover the first `limit` symbols; `total` is the full number.

## Dependencies and rescan

- `get_dependencies` takes exactly one of `symbol` and `layer` (both or neither is `MCP-101` with `params.detail = symbol or layer`). `direction` is `outgoing` (default: what the target uses) or `incoming`.
- With `symbol` (id or qualified name) the target is the symbol and everything it contains, so a namespace or a type gives the dependencies of the whole unit. References between the members of the unit are not dependencies. `dependencies` lists the symbols on the far side with the number of references (`UsageIndex::dependencies`); references from tests count only with `include_tests`. `layer` of a dependency is the layer that owns its namespace, `null` if none does.
- `violates_layering` is taken from the layer graph of the snapshot (`get_architecture`): a dependency breaks the layering if the recorded edge between the layer of the target and the layer of the other symbol does, in the direction of the dependency. Dependencies inside one layer, and dependencies of a target or to a symbol that no layer owns, never violate. `layers` adds the dependencies up per other layer and also carries `in_cycle`; `violations` counts the listed dependencies (before the cut) that break the layering.
- With `layer` (a name from `get_architecture`, otherwise `MCP-101` with `params.detail = layer`) the answer is the recorded layer graph for that layer: `layers` has one entry per other layer, `dependencies` is empty and `include_tests` has no effect.
- `limit` cuts `dependencies` (or, for a layer, `layers`); `total` is the full number and `notice` (`MCP-501`) says so. `layers` of a symbol answer is always computed from all dependencies.
- `rescan` runs an incremental scan of the project through the `ProjectScanner` port, implemented by the scanner of the application layer: only files with a changed hash are processed (blueprint 10.4). It needs the permission `scan`, checked before the tool name and the arguments are looked at, like `read` for the others; a caller without it gets `AUTH-401` and nothing is scanned. The answer names the new snapshot and the time the scan took; every other tool answers from that snapshot from then on. A failed scan keeps the code of the failed stage. The blueprint table lists an optional `paths` argument; it is not offered, because an incremental scan already restricts itself to changed files and a path argument would only add a path to validate (S-05). Two scans of the same project at the same time are not coordinated yet.

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

- `get_architecture` returns the layers and the measured dependencies. Layers come from the `[[layers]]` list of the configuration (`docs/cli.md`, ADR 0031); without it the list is empty. The allowed directions and the conventions are not part of the project model yet (see `docs/storage.md`, `violates_layering`); they join the answer with the layer rule of `check_conventions` (issue #45). The use case behind `check_conventions` and the tool exist (`docs/conventions.md`, `codeskimr check`); the layer rule joins it with issue #45.
- Paths from arguments (blueprint S-05): `check_conventions` takes paths, but only to select among the files of the project's own file list; a path is never opened, so `..` or an absolute path selects nothing. `rescan` takes no paths, so there is nothing to canonicalise.
- A qualified name shared by several symbols (overloads) answers with the lowest symbol id; use the id from `search_symbols` to be exact.
