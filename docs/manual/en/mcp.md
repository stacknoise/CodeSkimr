# MCP setup

MCP (Model Context Protocol) lets a code agent such as Claude Code or Cursor ask CodeSkimr about your code: search symbols, read signatures, find usages, check whether something similar already exists. Two ways to connect exist.

## 1. stdio (recommended)

The agent starts `codeskimr mcp` itself and talks to it over standard input and output. It serves **one project**, so the calls need no project argument.

1. Scan the project once ([Projects and scan](projects-and-scan.md)). Without a scan the command stops with `PRJ-201` (not registered) or `MCP-203` (registered, no snapshot).
2. Tell the agent how to start it. Generic configuration (Claude Desktop, Cursor and others use this shape):

```json
{
  "mcpServers": {
    "codeskimr": {
      "command": "codeskimr",
      "args": ["mcp", "--project", "/path/to/your/project"]
    }
  }
}
```

For Claude Code: `claude mcp add codeskimr -- codeskimr mcp --project /path/to/your/project`. Use the full path of `codeskimr` if it is not on the `PATH`. Without `--project` the current folder of the agent is used. Nothing but the protocol is written to standard output; errors go to standard error.

## 2. HTTP at `/mcp`

The running app (`codeskimr serve`) offers the same tools at `http://127.0.0.1:7421/mcp` (streamable HTTP, answers are JSON, no session is kept). One app serves all projects, so **every call names its project** (`project` argument; a call without it is `MCP-101`). The project ids are the ones of the project list.

Every request, also a pure read, needs the installation token as `Authorization: Bearer <token>`, because the answers contain the structure of your code and must not be readable by any program on the machine. `Host` and `Origin` are checked first. A client is configured with the URL and that header.

> **State:** the token is never printed by a command. How it reaches an agent configuration is part of the installation work and is not available yet. Use stdio until then.

## The tools

All tools only read, except `rescan` (and `get_project` runs the tool check, a few short processes). Every answer is structured JSON and says which snapshot it is based on (`snapshot.id`, `snapshot.age_seconds`).

| Tool | Use it to |
|---|---|
| `search_symbols` | find types and members by name (`query`, optional `kind`, `limit`) |
| `get_symbol` | read signature, documentation, visibility, lines, layer, metrics and members of one symbol (id or qualified name) |
| `find_usages` | who uses the symbol (`direction` `incoming`, default) or what it uses (`outgoing`); `depth`, `include_tests` |
| `impact_of_change` | which symbols, layers and public interfaces a change can reach |
| `get_context` | everything about one symbol in one call: signature, callers, dependencies, files a change can reach and notes; `budget` `short`, `medium` (default) or `long` says how much |
| `find_unused` | which symbols have no usage, with the assessment and the reason; symbols that the exception list of the project (`[[unused.ignore]]` in `.codeskimr.toml`) takes out are listed apart with their reason |
| `check_rules` | the architecture rules of the project with their violations, down to the place in the code; violations that are already held as old burdens are marked, only new ones fail the CI |
| `preview_rename` | what renaming a symbol would hit, without changing anything: the places that use it, symbols that already have the new name, and lines of the project folder (strings, configuration, documentation) that hold the old name |
| `get_dependencies` | what a symbol, namespace or layer depends on, and whether that breaks the layering |
| `get_architecture` | the layers and the dependencies between them |
| `find_similar_code` | does this already exist? Give a `symbol` or a `snippet` with its `language` |
| `find_constants` | find existing constants by name before adding a literal |
| `check_conventions` | run the convention checks over files of the project |
| `get_project` | languages of the project and the **tool check** of level 2: which indexers are ready, missing or too old, with purpose and install hint |
| `list_annotations` | the notes, tasks and critical marks that people put on symbols (`symbol`, `kind`, `status`; paged with `limit` and `offset`) |
| `get_alerts` | the alerts of critical symbols that changed, were removed or lost their place (`state`, `kind`; the open ones by default; paged). Agents can read them but not dismiss them |
| `rescan` | read changed files again (incremental); needs the right to scan. It never starts level 2 |

Good habits for agents: call `find_similar_code` and `find_constants` before writing new code and `check_conventions` before a commit.

## Reading results

- **Limits.** `limit` defaults to 20 and is held to at most 100. A cut result carries a `notice` with `MCP-501` and the applied limit; `total` says how many exist.
- **Certainty.** Every usage has a `confidence`, `certain` or `probable`. A probable one is a best guess that an agent should verify. Hits of level 1 are not checked by a compiler; see [Usages](usages.md#certainty-verified-unchecked-presumed). Every usage and every symbol of the impact also carries `verified` (true only if level 2 produced it); the answer counts `verified` and `unchecked` apart, lists verified ones first and sets `preliminary` to true while any listed one is unchecked. **An agent must not treat a preliminary answer as complete or exact** ([Language indexers](indexers.md)).
- **Unknown symbol** is `MCP-202`, never an empty list: "no usages" and "no such symbol" are different answers. A name shared by several overloads answers with the lowest id; take the id from `search_symbols` to be exact.
- **Errors** are normal tool results with `isError: true` and `{ "code", "description", "params" }`. The description is always English, for agents. See [Error codes](errors.md).

## Safety

The agent runs as you, the local owner, and may read the project's structure. Reading source text and starting a scan are separate permissions that the tools check. `check_conventions` only selects among the files of the project's own list; a path argument is never opened, so `..` or an absolute path selects nothing.

## The same answers on the command line

Scripts, CI and agents without MCP get the same answers from `codeskimr symbols <query>`, `codeskimr usages <symbol>`, `codeskimr impact <symbol>`, `codeskimr context <symbol>`, `codeskimr unused`, `codeskimr rules` and `codeskimr preview-rename <symbol> <new-name>`. Add `--json` for JSON (with the version of the format in `schema`, and `stale: true` if files changed since the scan, `false` if not, `null` if that cannot be told; only `false` means current); without it you get text. The project is the one of the current folder, or `--project <id or folder>`. The commands read the last scan; they never scan by themselves. See `docs/cli.md`.
