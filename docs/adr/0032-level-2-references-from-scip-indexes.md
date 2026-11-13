# 0032 – Level 2 references come from SCIP indexes and are checked against them

Status: proposed

## Context

Level 1 (blueprint 9.2) resolves names without type information. The accuracy measurement (`tools/accuracy`, `docs/accuracy.md`, issues #134 and #145) shows that this does not get reliable: with 150 sampled symbols, 35 % (Python) and 23 % (TypeScript) have exactly the usages the language tooling finds, and the first sample on a private C# project had 8 of 20 correct. Making level 1 better rebuilds parts of a compiler and has no end. The purpose of CodeSkimr is documentation and judging extensions and reworks (`impact_of_change`); that needs usages that are complete first and correct second. The product owner ranks missing usages above speed (13 November 2026).

Blueprint 9.3 already plans level 2 (SCIP indexes and Roslyn) "after 1.0, optional". This ADR moves it forward and fixes the rules.

## Decision

1. **Level 2 is a `ReferenceResolver` strategy that imports SCIP indexes** (blueprint 9.3, section 5). No other component changes. Indexers, by language (to be confirmed by the spike below):

   | Language | Indexer | Needs |
   |---|---|---|
   | C# | scip-dotnet (Roslyn) | .NET SDK, restorable project |
   | TypeScript, JavaScript | scip-typescript | Node, installed dependencies |
   | Python | scip-python | Node, the Python environment of the project |
   | Rust | `rust-analyzer scip` | Rust toolchain |
   | Java, Kotlin | scip-java | JDK, Gradle or Maven |

2. **Mapping.** An index occurrence is mapped to a CodeSkimr symbol by file and the range of its definition. Occurrences that map to no project symbol (framework, outside the project) are dropped. A symbol that SCIP knows and CodeSkimr does not, or the reverse, is counted and reported by the scan, not hidden.
3. **Replacement per file.** For every file that the index covers, the level 2 references replace all level 1 references of that file; they are `certain`. Files without index keep their level 1 references.
4. **Nothing is dropped silently.** Level 1 keeps every name match; it never discards a candidate. A match that no type evidence supports is stored as `probable`, and in a project or file without level 2 it is shown as unchecked. (This reverses the ambiguity rule of PR #133, which dropped matches and lowered the recall in TypeScript from 61 % to 42 %.) The exact wording of "unchecked" in UI and MCP is open question 1.
5. **Completeness is visible.** `find_usages` and `impact_of_change` report how many results are `certain` and how many are not, and whether level 2 ran for the files involved. An analysis without level 2 is labelled preliminary.
6. **Running an indexer is opt-in and security-sensitive.** Indexers restore dependencies and compile the project, which runs build scripts of the scanned code. Level 2 therefore runs only for a project whose `.codeskimr.toml` or user setting enables it (`level2 = true`) and only after the user confirmed it for that project. Limits: time, memory, output size, no execution for a Git URL that was not confirmed, the environment passed on is minimal. The pull request that adds the runner is labelled `security-sensitive` and gets a security review.
7. **Order of work.** Spike first (open question 2), then the import and merge (language independent), then per language: C#, Rust, TypeScript, Python, Java/Kotlin.
8. **Acceptance** (added to `docs/acceptance`): with level 2, recall of at least 99 % and at least 99 % correct among `certain` references on a sample of at least 150 symbols per language with `tools/accuracy`, which gets SCIP results as a second reference; the private C# sample of 20 symbols has at least 19 complete and without wrong usages. Without level 2: recall at least as high as before PR #133.
9. **Speed is secondary.** Level 1 answers first; level 2 completes later and replaces. A scan with level 2 may take minutes.

## Open questions

1. Wording and storage of "unchecked" for level 1 results (a new confidence value, or a flag `verified = false` next to `probable`). Proposal: a flag, so the stored confidence values stay `certain` and `probable`.
2. Spike result: which of the indexers run on the public corpora at all, how long they take, and how many of the symbols of the corpus they cover. Not yet run: the environment used for this draft could not install the .NET SDK. The spike starts with C# and Rust.
3. Protobuf decoding needs one new dependency (SCIP is protobuf); the choice is part of the pull request that adds the import (label `new-dependency`).

## Consequences

- Blueprint 9.3 and the roadmap (section "later") change: level 2 comes before 1.0, as an opt-in step. The blueprint is changed by the pull request that implements the import, with the label `blueprint-change`.
- Users need the toolchain of the language to get full accuracy; without it they get level 1, clearly marked.
- A new, security-relevant function (running build tools) with explicit consent.
- `tools/accuracy` becomes part of the acceptance criteria.
