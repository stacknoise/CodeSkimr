# Language indexers

> **State:** the indexers described here are decided in ADR 0032 and are built in separate packages. Until they are, every usage is "unchecked" ([Usages](usages.md#certainty-verified-unchecked-presumed)) and the usages screen says that no indexer is set up. This chapter describes how it works once they exist, so you know what to install.

## Why

Without type information, finding the usages of a name is guesswork: `Save()` may exist in forty classes. A compiler or language service knows which one a call means. CodeSkimr therefore imports the result of such a tool (a SCIP index) instead of rebuilding a compiler. Where an index exists for a file, its references replace the guesses for that file and are marked **verified**.

## You install them, CodeSkimr checks them

CodeSkimr **never downloads or installs** an indexer and never runs a package from the network. You install the indexer for each language you use, the way you install the compiler. The reason: the indexer needs your project's own dependencies and a matching SDK anyway (package restore, `node_modules`, a Python environment, the Gradle or Maven build), and only you can provide those.

| Language | Indexer (planned) | Also needed |
|---|---|---|
| C# | scip-dotnet | a .NET SDK that matches the project, restored packages |
| TypeScript, JavaScript | scip-typescript | Node.js, `node_modules` installed |
| Python | scip-python | a Python environment with the project's packages |
| Rust | `rust-analyzer scip` | the Rust toolchain |
| Java, Kotlin | scip-java | a JDK, the build tool of the project |

Go, C, C++, PHP and Ruby (basic tier) have no level 2; their usages stay unchecked.

The final list and versions are fixed by the spike of ADR 0032; this table may change.

## What CodeSkimr shows you

Whenever a project is loaded (added, scanned, reopened) CodeSkimr looks, per language of the project, whether the indexer and its toolchain are there and have a usable version. The result is stored with the project and shown in the UI, in the summary of `codeskimr scan` and over MCP:

- which languages have level 2 ready, and
- for every missing tool its name, what it is needed for and how to install it.

A missing tool **never stops a scan**: you get level 1, with every result marked unchecked. The check does not run any project code.

## Switching it on

Running an indexer runs build tools of your project, so it is opt-in:

- You enable it per project in your **user configuration** (or in your own project record) and confirm it once.
- A `level2` setting in the project's own `.codeskimr.toml` is **ignored**, so a repository you scan cannot switch on running its own build scripts.
- Only you, the local owner, can confirm and start an indexer. MCP, REST tokens, watch mode and CI cannot.

Limits apply to every run (time, memory, size of the output and of the index); an indexer that exceeds them is stopped as a process group. The environment it gets has no credentials of CodeSkimr. The network is not blocked in the first version, because restoring packages needs it; that is the reason for the one-time confirmation.

## After a change

An index counts for a file as long as its content hash is the one at indexing time. A file you changed falls back to level 1 and is marked unchecked until level 2 runs again, on the next scan or on demand. A corrupt or too large index is reported as an error and level 1 stays in place; it never breaks the scan.
