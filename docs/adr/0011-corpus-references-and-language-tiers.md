# 0011 – Corpus references and full-tier languages for 1.0

Status: proposed

## Context

Blueprint 17.2 requires a corpus project with expected results for every full-tier language. The product owner's private C# application is the C# reference, but it must never enter this public repository. The product owner has public repositories in several languages, but none in Go.

## Options

1. Keep seven full-tier languages and search for unrelated public corpus projects for each.
2. Use the product owner's own public repositories as corpus where they exist, use CodeSkimr itself for Rust, and move Go to the basic tier for 1.0.
3. Drop the corpus requirement for some languages.

## Decision

Option 2.

- C#: private reference (local only, never in the repository) plus one pinned public C# project for CI.
- TypeScript/JavaScript, Python, Kotlin, Java: a public repository of the product owner per language, pinned by commit.
- Rust: CodeSkimr itself at a pinned commit, chosen when the Rust language pack starts. Part of the expected values is checked by hand so that the scanner does not only confirm its own output.
- Go: basic tier in 1.0, full tier after 1.0 when needed.
- CI only uses public projects.

## Consequences

- Go full-tier work (WP-64) leaves the 1.0 plan.
- The corpus projects are chosen when each language pack starts (WP-04, WP-60 to WP-65); the exact repositories are recorded in `corpus/` with their commits.
- Updating the pinned commit of a corpus project is a deliberate change with updated expected values.
