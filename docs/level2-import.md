# Level 2 import: SCIP indexes (ADR 0033, package 5)

`crates/adapters/scip` (`codeskimr_scip`) turns a SCIP index into level 2 references. The mapping is a pure function over domain types; `ScipImporter` implements the `ReferenceImporter` port of the application layer, which the second pass of the scan (below) calls. The crate depends on `application`, `domain` and `codes`.

## Decoder

`decode_index(bytes, max_index_bytes)` and `read_index(path, max_index_bytes)` read the Protobuf format of Sourcegraph's `scip.proto` with a small hand-written reader (`wire.rs`): no `protoc`, no generated code, no new dependency. It reads only `Index.metadata.project_root`, `Document.relative_path` and `position_encoding`, and `Occurrence.range`, `symbol` and `symbol_roles`; every other field is skipped.

The index is untrusted input (ADR 0033 decision 4):

- The size is checked before the file is read, and the read stops one byte past `level2.max_index_bytes` (default 256 MiB). A larger index fails with `RES-501`.
- Nothing is allocated for input that has not been checked: strings are length-checked (paths 4 KiB, symbols 64 KiB) before they are copied, ranges hold at most four numbers, skipped groups may nest 8 deep, varints are strict (at most ten bytes, no bits beyond 64), and a SCIP symbol that occurs many times is stored once.
- The number of things is limited too, because a small file grows in memory: at most 1,000,000 documents, 16,000,000 occurrences and 4,000,000 distinct symbols (`consts.rs`). An occurrence takes about 13 bytes in the index and 24 in memory (a factor of about 2), and the text of every distinct symbol is held twice (list and lookup table) plus its parsed descriptor in the import, so budget roughly two to three times the index size for the decoded index and the import together. `read_index` reserves the file size once instead of growing the buffer.
- A bad range fails the index even on an occurrence without a symbol, which would be dropped anyway.
- Document paths are cleaned lexically (`/` as separator, no `.` and empty components) and refused if they are empty, start with a separator, name a drive or hold a `..` component (blueprint S-05). The import never opens a file by such a path; it only compares it with the paths of the snapshot.
- Truncated, malformed or out-of-range input is `RES-101`. The decoder does not panic; a test feeds it random and damaged bytes.

Columns of an occurrence are in the unit the document states (`PositionEncoding`, kept on the decoded document); scip-dotnet and scip-typescript count UTF-16 code units, the parsing adapter counts UTF-8 bytes. `convert_columns` turns the columns of a document that is not UTF-8 into byte columns before the mapping, using the lines of the file (`SourceText`, which never follows a link out of the project). A column inside a surrogate pair stands for the next character. If the text of a file cannot be read, or a column lies behind the end of its line, the occurrence keeps its columns. `read_index` refuses a link in place of the index file (issue #217).

## Mapping and replacing

`import_references(index, snapshot, min_mapped)` is a pure function. The snapshot is the files, symbols and references of one language.

1. **Definitions.** An occurrence with the definition bit (`symbol_roles & 1`) is mapped onto the smallest symbol of the same file whose span contains the position and whose name is the name of the last descriptor of the SCIP symbol (descriptor syntax of `scip.proto`: suffixes `/` `#` `.` `:` `!`, methods `name(+n).`, `[type parameter]`, `(parameter)`, backtick names with `` for a backtick). A qualified name in one backtick name matches by its last segment (`System.IDisposable.Dispose`), and `.ctor`, `<init>` and `<constructor>` match a project constructor, which is named like its type. Among symbols with the same span the one whose kind fits the descriptor wins, then the lowest id. `display_name` is not used.
2. **Overloads.** SCIP tells overloads apart and so does CodeSkimr (one symbol per declaration, same qualified name), so each overload follows from the position of its definition.
3. **Partial classes.** One SCIP symbol can have several definitions, one per file. The occurrence becomes one reference per CodeSkimr symbol of the type (`mapped` counts the occurrence once).
4. **References.** Every other occurrence of a mapped symbol becomes a reference from the smallest symbol of the file that contains the place to the mapped symbol, `certain` and `verified = true`. The kind is the kind of the level 1 reference at the same place (same start, else a span that holds the occurrence) if there is one, else by the target: callable is `call`, type is `type_use`, anything else `read`. The same place and pair is stored once. New ids follow the highest id of the input.
5. **Replacing per file.** For every file that an index document names (after cleaning the path), all level 1 references of the file are replaced. Files without an index document keep level 1.
6. **What is not a reference.** Local symbols (`local N`), parameters, type parameters, meta and macro descriptors, and namespaces without a definition are ignored. Occurrences of symbols the index does not define are library symbols and are dropped (`library`). Occurrences of symbols the index defines but that match no snapshot symbol (code generated by a macro, a name the language pack does not report) or that lie in no symbol are dropped as `unmapped`.
7. **Threshold.** The share `mapped / (mapped + unmapped)` must reach `level2.min_mapped` (default 0.5); library occurrences do not count, because most occurrences of a real project point into libraries (measured below: 50 %). An index that covers no file fails as well, and so does one whose covered documents hold uses of which none mapped (for example an indexer that never marks definitions). All fail with `RES-302`, and the caller keeps level 1. An occurrence that lies in no symbol of its file has no origin and counts as `unmapped`.
8. **`Self` in Rust.** rust-analyzer does not write `Self` as a use of the type but of its impl block: `.../impl#[Type]` (and `impl#[Type][Trait]` for a trait implementation), a symbol without a definition. Such a symbol maps to the type: first the symbol of the type in the same module (the text before `impl#` plus `Type#`), else the one project type of that name that the index defines. If two types of that name exist, the occurrence is ignored rather than guessed. The kind is the kind of the level 1 reference at the place, else `type_use`. Forms the import does not read (`impl#[Vec<u8>]`, `impl#[&Type]`) stay as before.
9. **Methods of trait implementations.** SCIP gives such a method the symbol `impl#[Type][Trait]name().` and marks its definition. For every such definition whose trait method the project defines (found as the one defined method `name` of a type descriptor `Trait`; ambiguous names give none), the import adds an `override` reference from the method to the method of the trait at the position of the name, `certain` and `verified`. Level 1 had this reference too, at the start of the function; because all references of a covered file are replaced, level 2 has to produce it itself. Not read from the index: the `Relationship` entries of `SymbolInformation` (issue #241 lists reading them as the cleaner way).

`ImportStats` has `mapped`, `library`, `unmapped`, `ignored`, `files_replaced` and `files_unmatched` (documents without a snapshot file, for example generated code that is not scanned).

## Measured on two real indexes

Not part of the tests (no foreign data in the repository). The comparison is against the index itself: for each SCIP symbol with a definition and uses, the lines of its uses in the index against the lines of the imported references (parameters left out).

| | C# (scip-dotnet) | Rust (rust-analyzer) |
|---|---|---|
| occurrences that mapped | 2142 | 25068 |
| library / unmapped / ignored | 2208 / 0 / 2732 | 25032 / 4315 / 22767 |
| share mapped of project symbols | 100 % | 85 % |
| symbols compared / exactly equal | 164 / 164 | 4774 / 4774 |
| recall / precision against the index | 100 % / 100 % | 100 % / 100 % |
| level 1 on the same symbols: recall / precision | 71 % / 97 % | 51 % / 20 % |

The 15 % that did not map in Rust are 77 types and macros that a `macro_rules!` invocation defines (`labeled_enum!`, `id_type!`), which the syntax tree does not show. Import time on the Rust index (9 MB) was about 30 ms for decoding and 30 ms for mapping.

## Limits

- The enclosing-symbol search is linear in the symbols of a file for every occurrence, so very large or single-line files are quadratic (issue #220).
- The index decides which files are covered; a file that the indexer did not see keeps its level 1 references.
- Columns are not converted between encodings (see above, issue #217).

## The second pass of the scan

After level 1 is stored, `Level2Pass` (application layer, `level2/pass.rs`) runs once per distinct indexer name. Languages that name the same indexer (TypeScript, JavaScript and TSX; Java and Kotlin) share one run and get one report each.

For each indexer, in this order: `authorize_run` (permission, enabled in the user configuration or the project record, confirmed); `IndexRunner::check` (tools installed in a usable version); `IndexRunner::run`; then a hash check of every file of the languages against the disk, which happens **after** the run because the run may take minutes. Files whose content changed keep level 1 and are counted as `stale`; if all changed, nothing is imported. The importer maps the index onto the files that passed, and `SnapshotStore::replace_references` replaces the references of exactly those files in one transaction; the new references get ids the pass reserved from the shared id sequence, so they cannot collide with rows of other snapshots; the transaction also updates the totals and rebuilds the read models.

Whatever goes wrong, the scan has finished and level 1 stays: the pass never fails the scan, a cancel kills the process group and leaves the references as they were, and the report names the reason per language (not permitted, not confirmed, tools missing, stale, cancelled, or the error code). `codeskimr scan` prints one line per language ("Level 2: ... checked" or "skipped (reason)"), `--json` carries the same data.

Rows are shared between snapshots that carry a file unchanged (ADR 0032). `replace_references` therefore treats every snapshot that holds a replaced file alike: the new references are visible from the oldest snapshot that holds the file, and the totals and read models (usage counts, layer graph, search) of all of them are built again, in the same transaction. Only the newest snapshot of a project can be asked for a replacement. A test with two snapshots that share a file pins this.

A file that is not valid UTF-8 is read lossily by `FolderText` (invalid bytes become a three-byte character), so byte columns behind such a byte can be off; the hash check does not help there. An occurrence that cannot be placed in the text of its line is dropped, not kept with a wrong column. `SnapshotCreated` is published as soon as level 1 is stored and once more after level 2 if the totals changed, so UI, MCP and watch mode do not wait for the indexers. Every indexer group hashes the whole project once more after its run; hashing only the files of the group is tracked in an issue. Watch mode runs level 2 with its first scan only; later scans of the session are level 1 (`Job::without_level2`).
