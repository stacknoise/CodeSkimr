# Convention check (WP-35)

Blueprint 10.4 (`check_conventions`), 10.2 (`codeskimr check`), 11.6 (E-04) and 17.4 (dogfooding). The check reads files the way a scan does and reports where they break the code rules of blueprint section 3. It uses the same `CHK` error codes as agents and CI (`errors/error-codes.toml`).

## What is checked

| Code | Rule | Status |
|---|---|---|
| `CHK-104` `function_too_long` | R-02: at most `conventions.max_function_lines` code lines per function (default 40) | checked |
| `CHK-102` `duplicate_block` | R-01: two functions whose syntax fingerprints are at least `conventions.duplicate_similarity_percent` similar (default 85) | checked, within the checked files |
| `CHK-101` `literal_found` | R-06 | not checked yet (#93): the language packs do not report literals. The Dylint lint `no_literals` covers Rust in CI |
| `CHK-103` `layer_violation` | R-04 | not checked yet: the project model has no allowed directions (issue #45) |
| `CHK-105` `complexity_too_high` | R-02: cognitive complexity | not checked yet (#93): the packs count decision points (cyclomatic), not cognitive complexity |

The report always lists the rules it did not check (`not_checked`), so a clean report is never read as "all rules are met". The limit of 5 parameters (R-02) has no code in the catalogue and is not checked.

## Where it lives

- `codeskimr_domain::similar_pairs` finds all similar pairs in a set of fingerprinted bodies. It works on an inverted index of the hashes, so the cost follows the number of shared hashes, not the square of the number of functions. Its value is the same Jaccard similarity as `Fingerprints::similarity` (a test compares them), and it is the function `find_similar_code` will use as well (blueprint 10.4: one implementation).
- `codeskimr_application::conventions::ConventionChecker` is the use case. It takes the ports `FileDiscovery` and `FileAnalyzer` that a scan uses, so both see the same files (`scan.excluded_folders`, the installed language packs). It needs `read_source` (`AUTH-401` otherwise) and stores nothing.
- `codeskimr_cli` renders the report (`check.rs`), `crates/app/src/check.rs` runs it.

## How a request is read

- `paths` select files from the project's own file list: a file, or a folder with everything below it; no path or `.` selects everything. Separators may be `/` or `\`, a leading `./` is dropped. A path is never opened itself, so `..` or an absolute path outside the root selects nothing (blueprint S-05); it can never widen the selection.
- The files are analysed afresh, so the check sees edits that no scan has seen. A file that cannot be analysed (no language pack, parse timeout `PRS-501`) does not stop the check; it is listed in `unchecked` with its code.
- Duplicates are found among the selected files. To find a copy of code that is outside the selection, select the whole project (CI does). Comparing against the stored project needs persisted fingerprints (#94, with the MCP tool in WP-37).
- A function reports at most one duplicate finding: on the later function in path and line order, naming the earlier one and the similarity (rounded down to a whole percent). Findings are ordered by file, line and code.
- Length is the code lines of the function (ADR 0015: no blank or comment-only lines). A nested function counts for itself and for the function around it.

## Configuration

```toml
[conventions]
max_function_lines = 40
duplicate_similarity_percent = 85
```

Both must be positive, the second at most 100 (`CFG-101`). The similarity itself is tuned by `[similarity]` (n-gram size, window size, minimum tokens). Functions with fewer than `similarity.min_tokens` tokens have no fingerprints and are never reported as duplicates.

## Not part of this package

- The MCP tool `check_conventions` and the check of a diff (the caller passes the changed paths) are WP-37.
- `CHK-101`, `CHK-103` and `CHK-105` as listed above, and the matching `get_architecture` conventions (#45).
