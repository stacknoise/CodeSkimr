# 0020 – Basic-tier grammars: compiled in behind a feature, loaded on first use

Status: proposed

## Context

Blueprint 8.2 says basic-tier grammars are "loaded on demand as language packs, which keeps the binary small". Blueprint 8.1 allows a grammar to be "built in, or a dynamically loaded library". Loading a Tree-sitter grammar from a shared library needs `unsafe` (a foreign function table), and rule R-08 forbids `unsafe` outside `codeskimr_secrets::memory`. A dynamic loader would also be a new trust boundary: a pack folder could then run native code, and the security review (blueprint 12) has not looked at that.

## Options

1. A loader for shared libraries (`libloading` or similar) in the parsing adapter. Needs an exception to R-08, a new dependency, a security review and a build and signing story for one library per platform and language.
2. Compile the basic grammars into the binary, behind the cargo feature `basic-grammars`, and read their packs on first use.
3. Compile every grammar in and load every pack at start (what WP-24 does for the full tier).

## Decision

Option 2.

- The grammars of the basic tier are optional dependencies of `codeskimr_parsing`, switched on by the feature `basic-grammars` (default on). Without the feature only the full-tier grammars exist, and a basic pack fails to load with `PRS-301`. A slim build is therefore possible, a larger one is the default.
- "On demand" means at run time: `LanguagePacks::load` reads only `language.toml` of a basic pack (id, tier, extensions, enough to route files and to show the tier). Grammar and queries are compiled when the first file of that language is analyzed. A scan of a project without Go never builds the Go queries. A broken basic pack is reported with `PRS-301` at that moment, a broken full pack when the packs are loaded.
- The only Rust per language is one line in the grammar registry (name to grammar). Everything else of a basic pack is data in `languages/<lang>/`, as blueprint 8.2 requires.
- Basic packs in 1.0: Go, C, C++, Ruby and PHP. Go is the planned full-tier language after 1.0 (ADR 0011). ABAP stays out of 1.0.
- Dynamic libraries stay possible later by their own ADR (R-08 exception, security review). The `grammar` key of `language.toml` does not change.

## Consequences

- Blueprint 8.2 is reworded: basic grammars are compiled in behind a cargo feature and loaded lazily. The size budget of section 16 is checked by the release build.
- Every added basic language is a dependency (`new-dependency` label, licence check by `cargo deny`), until a dynamic loader exists.
- Pack validation for basic packs happens on first use; the golden test loads every pack in `languages/`, so a broken pack still fails CI.
