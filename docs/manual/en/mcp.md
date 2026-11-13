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

All tools only read, except `rescan`. Every answer is structured JSON and says which snapshot it is based on (`snapshot.id`, `snapshot.age_seconds`).

| Tool | Use it to |
|---|---|
| `search_symbols` | find types and members by name (`query`, optional `kind`, `limit`) |
| `get_symbol` | read signature, documentation, visibility, lines, layer, metrics and members of one symbol (id or qualified name) |
| `find_usages` | who uses the symbol (`direction` `incoming`, default) or what it uses (`outgoing`); `depth`, `include_tests` |
| `impact_of_change` | which symbols, layers and public interfaces a change can reach |
| `get_dependencies` | what a symbol, namespace or layer depends on, and whether that breaks the layering |
| `get_architecture` | the layers and the dependencies between them |
| `find_similar_code` | does this already exist? Give a `symbol` or a `snippet` with its `language` |
| `find_constants` | find existing constants by name before adding a literal |
| `check_conventions` | run the convention checks over files of the project |
| `rescan` | read changed files again (incremental); needs the right to scan |

Good habits for agents: call `find_similar_code` and `find_constants` before writing new code and `check_conventions` before a commit.

## Reading results

- **Limits.** `limit` defaults to 20 and is held to at most 100. A cut result carries a `notice` with `MCP-501` and the applied limit; `total` says how many exist.
- **Certainty.** Every usage has a `confidence`, `certain` or `probable`. A probable one is a best guess that an agent should verify. Hits of level 1 are not checked by a compiler; see [Usages](usages.md#certainty-verified-unchecked-presumed). The level 2 mark ([Language indexers](indexers.md)) reaches MCP with the indexer packages.
- **Unknown symbol** is `MCP-202`, never an empty list: "no usages" and "no such symbol" are different answers. A name shared by several overloads answers with the lowest id; take the id from `search_symbols` to be exact.
- **Errors** are normal tool results with `isError: true` and `{ "code", "description", "params" }`. The description is always English, for agents. See [Error codes](errors.md).

## Safety

The agent runs as you, the local owner, and may read the project's structure. Reading source text and starting a scan are separate permissions that the tools check. `check_conventions` only selects among the files of the project's own list; a path argument is never opened, so `..` or an absolute path selects nothing.
