# 0055 – Level 2 creates symbols that level 1 does not know

Status: accepted

Supplements ADR 0033, decision 2 ("occurrences that map to no symbol are dropped"): for definitions that the index states and level 1 lacks, level 2 now creates the symbol instead of dropping the uses. The other decisions of ADR 0033 stay.

## Context

Level 1 reads the syntax tree of one file. Code that a macro generates is not in the tree, so level 1 has no symbol for it. In the Rust reference corpus (CodeSkimr itself) 77 definitions are of that kind: 24 types and 8 enums that `id_type!(SymbolId)`, `text_type!` and `labeled_enum! { SymbolKind { ... } }` define, the 40 enum variants of those enums, and the 5 `macro_rules!` definitions (which this ADR does not create as symbols, see decision 2: 72 symbols). rust-analyzer knows all of them and writes them to the SCIP index with kind, display name, signature, documentation and the range of the whole macro call (`enclosing_range`).

The import (ADR 0033) maps an occurrence onto a symbol that level 1 stored. For these definitions there is none, so every use is dropped: 1498 of 8792 project uses in the Rust index, 17.0 % (the most used types of the domain crate among them: `SymbolKind` 219 uses, `SymbolId` 185, `ReferenceKind` 113). Level 1 has no target for them either. Until now the accuracy measurement could not see this, because it draws its sample from the symbols of level 1. PR #375 (not yet on `main` when this ADR was written) adds `without_level1_symbol` to `tools/accuracy/measure.py` and describes the gap in `docs/accuracy.md`; it makes the gap visible, it does not close it.

The figure 17.0 % is an upper bound for the macro case, not a proven attribution: `unmapped` also counts uses that lie in no symbol of their file and names that the language pack does not report. The 77 definitions are the cause that was examined, the split of the 1498 uses was not measured. The first task of the implementation is therefore to break the 1498 down by the definition they point to, and the acceptance below is stated for what that shows. The other languages show it too, with other causes that have not been examined: TypeScript 6.9 %, Python 14.2 %, Java 0.7 % of the project uses are dropped for want of a symbol.

The aim of the product owner is usages that are nearly as exact as the compiler. A type that is used in 17 % of all places and has no usages page is a hole in that aim.

## Options

A. **Teach level 1 the macros of the project.** A query of the Rust pack per macro (`id_type!`, `labeled_enum!`) that reads the arguments as a type or an enum. It is a pattern per project macro, it breaks with every new macro or change of its grammar, and it can only guess what the macro generates. Rejected.

B. **Level 2 creates the symbols** that the index defines and level 1 does not know, from what the indexer states (kind, name, range, signature, documentation). The import knows these facts for certain; nothing is guessed. This ADR.

C. **Document the gap and measure it** (`without_level1_symbol`, PR #375). Needs no decision and is done, but closes nothing. It stays as the interim state and as the measurement of B.

D. **Expand the macros before scanning** (`cargo expand`). Runs the build and every procedural macro of the project as part of level 1, which is what level 2 consent is for (ADR 0033, decisions 9 to 12), and the expanded code has no stable positions in the files of the user. Rejected.

E. **Attach the uses to the enclosing module or to the macro call.** There is no symbol for a macro call, and a module is the wrong target: its usages page would list uses of a type. Rejected.

F. **Store the references against an external key symbol** (the SCIP symbol string) without creating a full symbol. It is lighter and needs no kind table, but a reference without a symbol has no usages page, no search entry, no type summary and no place in the layer graph, so the hole in the aim stays where users look. It also needs a second kind of reference target in the storage. Rejected.

## Decision

Option B, in these parts.

1. **A symbol has an origin.** `Symbol` gets `origin`: `syntax` (level 1, as today) or `index` (created by level 2). Stored in the column `origin` of `symbol_rows` (`TEXT NOT NULL DEFAULT 'syntax'`, in the next free migration at the time of the implementation). REST, MCP and export carry it. A symbol carries only its origin: `verified` stays a property of a reference (blueprint 9.4), and a symbol of origin `index` has no metrics. The references to it from level 2 are verified like every level 2 reference.
2. **What is created.** For every definition occurrence of a covered document whose symbol the mapping cannot match (no symbol of level 1 of that file with that name that contains the definition) and that is a project symbol (the index defines it, it is not local, not a parameter or type parameter, not a namespace that is only a module path), one symbol is created. Macros (`macro_rules!` definitions) are not created: the Rust pack leaves them out of the symbols on purpose, and blueprint 6.1 has no kind for them; that would be a new kind and needs its own decision (question 3). Their uses stay unmapped.
   - `kind`: from `SymbolInformation.kind` through a table of the language pack (`[indexer.symbol_kinds]`, SCIP kind name to the vocabulary of 6.1). A kind that is not in the table creates nothing. No kind is derived from the shape of the symbol text.
   - `name`: the display name, which must equal the last descriptor of the symbol; else nothing is created.
   - `qualified name`: built like level 1 would build it for the same place: the parent's qualified name plus the name (ADR 0036 for namespaced languages).
   - `parent`: the symbol that the SCIP descriptor path names, if it is a symbol of the snapshot (also a symbol created in the same import, so the variants of a created enum belong to it); else the innermost symbol of level 1 that contains the definition; else nothing is created. The parent must lie in the same file as the created symbol; a definition whose parent is in another file creates nothing, its uses are dropped and counted as `unmapped`.
   - `span`: the `enclosing_range` of the definition if the index has one, else its range. For a macro call this is the call.
   - `signature` and `doc` from the signature documentation and the documentation of the index; `visibility` stays empty (unknown), `static` false.
   - File: the file of the document. A symbol of origin `index` therefore lives and dies with that file, like every shared row (ADR 0032).
3. **Nothing is mixed.** Level 2 never changes, moves or deletes a symbol of origin `syntax`; the ids of level 1 stay as they are. It replaces the symbols of origin `index` of a file as a whole, in the same transaction as the references of that file (`ReferenceReplacement` becomes `Level2Replacement`: files, symbols, references). New ids come from the sequence that the pass reserves for references today. A created symbol is created only where level 1 has none for that definition, so the two sets of a file never overlap.
4. **Rescan.** A file that is carried unchanged keeps its symbols of origin `index` together with its level 2 references; a changed file gets level 1 symbols and references only (its old symbols of origin `index` are not carried, as no row of a changed file is). The level 2 pass of the next command line scan creates them again. The references of level 2 in unchanged files that point to a created symbol of a changed file are dropped in the same step (their target is gone) and counted; the next level 2 pass restores them. The scan by MCP or UI never runs level 2 (ADR 0033, decision 13): after it, the created symbols of the changed files are missing until the next command line scan, so the usages page of such a type can differ between two scans. This is accepted and documented.
5. **Level 1 stays deterministic.** The resolver of level 1 ignores symbols of origin `index` when it resolves names, so a level 1 scan gives the same result whether level 2 ran before or not.
6. **Read models.** `search_index`, `usage_counts`, `type_summary` and `layer_graph` are rebuilt as today from the symbols and references of the snapshot, so created symbols appear in search, in the usages page, in the type summary and in the layer graph. Metrics, fingerprints (duplicates) and the code rules of `check` skip them: there is no code of their own to measure. The coupling (ADR 0037) counts references and so includes them.
7. **Limits (the index is untrusted).** `level2.max_created_symbols` (default in `config/default.toml`) per index; an index that wants more fails as a whole, with a coded failure: no symbol and no reference of that import is stored, and level 1 stays (the same way the other limits of the import act, ADR 0033). The kind table (`[indexer.symbol_kinds]`, in the indexer section of the pack because it describes what that indexer reports, not a setting of level 2) comes from the language pack through the existing catalog port that already gives the tool plan (`codeskimr_parsing` implements it), reaches the `scip` adapter as a field of `ImportRequest` in `application`, and `Level2Replacement` is a type of the `application` port that `codeskimr_storage` implements. The adapters keep to `application`, `domain` and `codes`; there is no new dependency edge (blueprint 4.2). Names pass the validation of `SymbolName`, qualified names that of `QualifiedName`. Only documents of the snapshot create symbols. The creation is off for languages whose pack has no `[indexer.symbol_kinds]`; the first user is the Rust pack.
8. **The threshold** `level2.min_mapped` counts a created symbol's uses as mapped.
9. **UI.** The explorer and the usages page list created symbols like others. A visible marker is not part of this decision; it needs a mock first (blueprint 15.1) and is a question to the product owner. Until then the origin is in the API and in the MCP answers only.

## Measurement

- The accuracy measurement draws its sample from all symbols including those of origin `index`, and `without_level1_symbol` is expected to fall for Rust. Independence is limited: the reference tool (rust-analyzer over LSP) and the index (`rust-analyzer scip`) are the same engine, so for macro-generated symbols a figure of 99 % is partly circular (ADR 0033, decision 14 asks for a reference that does not depend on the index). The set is small (72 symbols), so it is checked completely instead of by sample: every created symbol against the source text of its macro call (kind, name, parent, variants), plus a hand-made sample of uses against the source. The remaining limit, a shared mistake of rust-analyzer, is named as a risk.
- Acceptance: Rust on the reference corpus, precision and recall at least 99 % over the symbols of origin `index`, no change for the other symbols, and the share of dropped uses (`unmapped`) falls to the part that the breakdown of the first task attributes to something else than these 72 symbols, and under 1 % if the breakdown shows that nothing else is in it.
- Tests: golden import tests with a macro-generated struct, enum and variants (kinds, parents, spans, replacement on a second import, an index that wants too many symbols, a name that differs from its descriptor), a storage test for the replacement (the transaction, STO-901 checks, prune of the rows with the file), a rescan test (carried and changed file), and a test that level 1 results do not depend on created symbols.

## Consequences

- Blueprint changes with the implementing pull requests (label `blueprint-change`): 6.1 (`Symbol` gets `origin`), 6.2 (the column, the replacement), 9.3 and 9.4 (level 2 creates symbols; verified), 10.4 (MCP fields), 17.2 (acceptance over symbols of origin `index`); `docs/storage.md`, `docs/level2-import.md`, `docs/accuracy.md`.
- Storage: a migration, `Level2Replacement` in the storage port, a check that a created symbol's file and parent are in the snapshot, and the prune of the rows already follows the file.
- The domain `Symbol` changes (new field), so every place that builds one gets `origin: Syntax`; a mechanical change in tests and adapters.
- New error codes with messages in de and en (index wants too many symbols), registered in `errors/error-codes.toml`.
- A created symbol is as exact as the index. A wrong index (a faulty indexer) now creates wrong symbols and not only wrong references; the limits above and the origin marker bound the damage, and the measurement over the reference corpus is the check.

## Risks

- **Parents and qualified names:** an index that nests symbols otherwise than the pack does (Rust variants are type descriptors, a project enum member is not) must be mapped by the kind table and the descriptor path; a wrong parent breaks the type summary of that type. The golden tests pin it per language.
- **Names that clash:** a created symbol may have the qualified name of a level 1 symbol (a macro that generates a type of the same name in another module of the same crate path). Both rows exist; search and the layer graph treat them as two symbols, as they do for two level 1 symbols of one qualified name today.
- **Other languages:** TypeScript and Python lose 6.9 % and 14.2 % of their uses for reasons that were not examined (dynamic definitions, decorators, generated code). The mechanism applies to them only where their pack has a kind table, and only after the cause of the gap is known to be a missing symbol. That is a task of its own.
- **Consent:** the symbols exist only after level 2 ran, so a project without it keeps the gap. This is documented and measured, not hidden.

## Decisions of the product owner

The product owner accepted this ADR on 15 November 2026 with the recommended answers to the three questions:

1. **Marker in the UI:** none for now. A created symbol carries its origin only in the API and in the MCP answers. A marker needs a mock first (blueprint 15.1).
2. **Languages:** Rust only in the first step. Other languages only after the cause of their gap was measured and their pack has a kind table.
3. **`macro_rules!`:** not created as symbols (72 symbols instead of 77). A symbol kind for macros would need its own decision.
