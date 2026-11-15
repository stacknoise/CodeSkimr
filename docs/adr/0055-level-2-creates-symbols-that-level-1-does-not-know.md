# 0055 – Level 2 creates symbols that level 1 does not know

Status: proposed

## Context

Level 1 reads the syntax tree of one file. Code that a macro generates is not in the tree, so level 1 has no symbol for it. In the Rust reference corpus (CodeSkimr itself) 77 definitions are of that kind: 24 types and 8 enums that `id_type!(SymbolId)`, `text_type!` and `labeled_enum! { SymbolKind { ... } }` define, the 40 enum variants of those enums, and the 5 `macro_rules!` definitions. rust-analyzer knows all of them and writes them to the SCIP index with kind, display name, signature, documentation and the range of the whole macro call (`enclosing_range`).

The import (ADR 0033) maps an occurrence onto a symbol that level 1 stored. For these definitions there is none, so every use is dropped: 1498 of 8792 project uses in the Rust index, 17.0 % (the most used types of the domain crate among them: `SymbolKind` 219 uses, `SymbolId` 185, `ReferenceKind` 113). Level 1 has no target for them either. The accuracy measurement cannot see this, because it draws its sample from the symbols of level 1 (`without_level1_symbol` in `tools/accuracy/measure.py` and `docs/accuracy.md` makes the gap visible; PR #375). The other languages show it too, with other causes that have not been examined: TypeScript 6.9 %, Python 14.2 %, Java 0.7 % of the project uses are dropped for want of a symbol.

The aim of the product owner is usages that are nearly as exact as the compiler. A type that is used in 17 % of all places and has no usages page is a hole in that aim.

## Options

A. **Teach level 1 the macros of the project.** A query of the Rust pack per macro (`id_type!`, `labeled_enum!`) that reads the arguments as a type or an enum. It is a pattern per project macro, it breaks with every new macro or change of its grammar, and it can only guess what the macro generates. Rejected.

B. **Level 2 creates the symbols** that the index defines and level 1 does not know, from what the indexer states (kind, name, range, signature, documentation). The import knows these facts for certain; nothing is guessed. This ADR.

C. **Document the gap and measure it** (`without_level1_symbol`, PR #375). Needs no decision and is done, but closes nothing. It stays as the interim state and as the measurement of B.

D. **Expand the macros before scanning** (`cargo expand`). Runs the build and every procedural macro of the project as part of level 1, which is what level 2 consent is for (ADR 0033, decisions 9 to 12), and the expanded code has no stable positions in the files of the user. Rejected.

E. **Attach the uses to the enclosing module or to the macro call.** There is no symbol for a macro call, and a module is the wrong target: its usages page would list uses of a type. Rejected.

## Decision

Option B, in these parts.

1. **A symbol has an origin.** `Symbol` gets `origin`: `syntax` (level 1, as today) or `index` (created by level 2). Stored in the column `origin` of `symbol_rows` (`TEXT NOT NULL DEFAULT 'syntax'`, schema version 5, migration after the last one at the time of the implementation). REST, MCP and export carry it; a symbol of origin `index` is verified by definition and has no metrics.
2. **What is created.** For every definition occurrence of a covered document whose symbol the mapping cannot match (no symbol of level 1 of that file with that name that contains the definition) and that is a project symbol (the index defines it, it is not local, not a parameter or type parameter, not a namespace that is only a module path), one symbol is created:
   - `kind`: from `SymbolInformation.kind` through a table of the language pack (`[level2.symbol_kinds]`, SCIP kind name to the vocabulary of 6.1). A kind that is not in the table creates nothing. No kind is derived from the shape of the symbol text.
   - `name`: the display name, which must equal the last descriptor of the symbol; else nothing is created.
   - `qualified name`: built like level 1 would build it for the same place: the parent's qualified name plus the name (ADR 0036 for namespaced languages).
   - `parent`: the symbol that the SCIP descriptor path names, if it is a symbol of the snapshot (also a symbol created in the same import, so the variants of a created enum belong to it); else the innermost symbol of level 1 that contains the definition; else nothing is created.
   - `span`: the `enclosing_range` of the definition if the index has one, else its range. For a macro call this is the call.
   - `signature` and `doc` from the signature documentation and the documentation of the index; `visibility` stays empty (unknown), `static` false.
   - File: the file of the document. A symbol of origin `index` therefore lives and dies with that file, like every shared row (ADR 0032).
3. **Nothing is mixed.** Level 2 never changes, moves or deletes a symbol of origin `syntax`; the ids of level 1 stay as they are. It replaces the symbols of origin `index` of a file as a whole, in the same transaction as the references of that file (`ReferenceReplacement` becomes `Level2Replacement`: files, symbols, references). New ids come from the sequence that the pass reserves for references today. A created symbol is created only where level 1 has none for that definition, so the two sets of a file never overlap.
4. **Rescan.** A file that is carried unchanged keeps its symbols of origin `index` together with its level 2 references; a changed file gets level 1 symbols and references only (its old symbols of origin `index` are not carried, as no row of a changed file is). The level 2 pass of the next command line scan creates them again. The scan by MCP or UI, which never runs level 2 (ADR 0033, decision 13), changes nothing here.
5. **Level 1 stays deterministic.** The resolver of level 1 ignores symbols of origin `index` when it resolves names, so a level 1 scan gives the same result whether level 2 ran before or not.
6. **Read models.** `search_index`, `usage_counts`, `type_summary` and `layer_graph` are rebuilt as today from the symbols and references of the snapshot, so created symbols appear in search, in the usages page, in the type summary and in the layer graph. Metrics, fingerprints (duplicates) and the code rules of `check` skip them: there is no code of their own to measure. The coupling (ADR 0037) counts references and so includes them.
7. **Limits (the index is untrusted).** `level2.max_created_symbols` (default in `config/default.toml`) per index; an index that wants more creates none and keeps level 1, reported as a coded failure. Names pass the validation of `SymbolName`, qualified names that of `QualifiedName`. Only documents of the snapshot create symbols. The creation is off for languages whose pack has no `[level2.symbol_kinds]`; the first user is the Rust pack.
8. **The threshold** `level2.min_mapped` counts a created symbol's uses as mapped.
9. **UI.** The explorer and the usages page list created symbols like others. A visible marker is not part of this decision; it needs a mock first (blueprint 15.1) and is a question to the product owner. Until then the origin is in the API and in the MCP answers only.

## Measurement

- The accuracy measurement draws its sample from all symbols including those of origin `index` (the reference tool of Rust, rust-analyzer over LSP, finds their references), and `without_level1_symbol` is expected to fall below 1 % for Rust. Acceptance of the change: Rust on the reference corpus, precision and recall at least 99 % over the symbols of origin `index`, no change for the other symbols, and the share of dropped uses (`unmapped`) from 17.0 % to under 1 %.
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

## Questions for the product owner

1. Does a created symbol get a visible marker in the UI (a mock is needed), or only the origin in API and MCP for now?
2. Is Rust the only language of the first step, as proposed?
