# Convention check (WP-35)

Blueprint 10.4 (`check_conventions`), 10.2 (`codeskimr check`), 11.6 (E-04) and 17.4 (dogfooding). The check reads files the way a scan does and reports where they break the code rules of blueprint section 3. It uses the same `CHK` error codes as agents and CI (`errors/error-codes.toml`).

## What is checked

| Code | Rule | Status |
|---|---|---|
| `CHK-104` `function_too_long` | R-02: at most `conventions.max_function_lines` code lines per function (default 40) | checked |
| `CHK-102` `duplicate_block` | R-01: two functions whose syntax fingerprints are at least `conventions.duplicate_similarity_percent` similar (default 85) | checked, within the checked files and (MCP) against the stored project |
| `CHK-101` `literal_found` | R-06 | not checked yet (#93): the language packs do not report literals. The Dylint lint `no_literals` covers Rust in CI |
| `CHK-103` `layer_violation` | R-04 | not checked yet: the project model has no allowed directions (issue #45) |
| `CHK-105` `complexity_too_high` | R-02: cognitive complexity | not checked yet (#93): the packs count decision points (cyclomatic), not cognitive complexity |

The report always lists the rules it did not check (`not_checked`), so a clean report is never read as "all rules are met". The limit of 5 parameters (R-02) has no code in the catalogue and is not checked.

## Where it lives

- `codeskimr_domain::similar_pairs` finds all similar pairs in a set of fingerprinted bodies. It works on an inverted index of the hashes, so bodies that share no hash are never compared; a hash that more than `conventions.duplicate_max_hash_sharing` functions share (default 100: getters, constructors, boilerplate) does not nominate pairs, because it would cost the square of that number. A nominated pair is still measured over all its hashes. Measured with the benchmark `domain/similar_pairs_boilerplate` (3000 functions that all share 40 tokens of boilerplate): about 8 ms with the limit; without it the same run did not finish within ten minutes. Its value is the same Jaccard similarity as `Fingerprints::similarity` (a test compares them), and it is the function `find_similar_code` will use as well (blueprint 10.4: one implementation).
- `codeskimr_application::conventions::ConventionChecker` is the use case. It takes the ports `FileDiscovery` and `FileAnalyzer` that a scan uses, so both see the same files (`scan.excluded_folders`, the installed language packs). It needs `read_source` (`AUTH-401` otherwise) and stores nothing.
- `codeskimr_cli` renders the report (`check.rs`), `crates/app/src/check.rs` runs it.

## How a request is read

- `paths` select files from the project's own file list: a file, or a folder with everything below it; no path or `.` selects everything. Separators may be `/` or `\`, a leading `./` is dropped. A path is never opened itself, so `..` or an absolute path outside the root selects nothing (blueprint S-05); it can never widen the selection. A named path that selects no file is listed in `unmatched`, and `codeskimr check` exits with 1 then, so a typo cannot pass as a clean check.
- The files are analysed afresh, so the check sees edits that no scan has seen. A file that cannot be analysed (no language pack, parse timeout `PRS-501`) does not stop the check; it is listed in `unchecked` with its code.
- Duplicates are found among the selected files. To find a copy of code that is outside the selection, select the whole project (CI does). `find_similar_code` (WP-37, `docs/mcp.md`) compares against the stored project: the fingerprints are persisted with every snapshot (`docs/storage.md`). `check_conventions` over MCP does the same: a function that no other checked function copies is also compared with the stored functions of the newest snapshot. Stored functions in the checked files themselves (their older versions) and in `test_folders` are never an original. Without a snapshot, or in `codeskimr check` on the command line, only the checked files are compared.
- A function reports at most one duplicate finding: on the later function in path and line order, naming the earlier one and the similarity (rounded down to a whole percent). Findings are ordered by file, line and code.
- Length is the code lines of the whole declaration, signature included (ADR 0015: no blank or comment-only lines). Clippy's `too_many_lines` in CI counts from the opening brace of the body, so a function with a signature over several lines can be reported here (`CHK-104`) with 41 to 43 lines while clippy accepts it. CI stays the authority; the difference is at most the extra signature lines. A nested function counts for itself and for the function around it. A function and one nested inside it are never reported as duplicates of each other.

## Configuration

```toml
[conventions]
max_function_lines = 40
duplicate_similarity_percent = 85
duplicate_min_lines = 8
duplicate_max_hash_sharing = 100
test_folders = ["tests", "benches"]
```

The first three must be positive, the second at most 100 (`CFG-101`). `duplicate_min_lines`: a function with fewer code lines is never compared for duplicates (three-line helpers and parallel test functions are alike by nature). Measured on this repository: 5 lines give 21 duplicate findings, 8 give 9, and what stays are mostly real copies. `jscpd` in CI is stricter by tokens, not by lines. `test_folders`: files below a folder with one of these names (including the fixtures in `languages/*/tests/`) are skipped by all convention checks and not counted in `files_checked`. Tests inside `src` (`#[cfg(test)]` modules) are not recognised. The similarity itself is tuned by `[similarity]` (n-gram size, window size, minimum tokens). Functions with fewer than `similarity.min_tokens` tokens have no fingerprints and are never reported as duplicates.

## The MCP tool

`check_conventions` (WP-37) runs this check for a registered project: `paths` (the files of a change; none means the whole project) and `limit`. It needs a project that is a folder on this machine (`PRJ-101` for a Git source, whose files live in the cache) and the permissions `read` and `read_source`. The answer lists `findings` (code, description, path, symbol, one-based lines, and `lines`/`max_lines` or `duplicate_of`/`similarity_percent`), `total`, `files_checked`, `unchecked`, `unmatched` and `not_checked`; findings beyond the limit are cut with the notice `MCP-501`. Paths are never opened (see above), so the tool cannot be used to read outside the project (blueprint S-05).

## Not part of this package

- The duplicate check against the stored project in `codeskimr check` (it has no store; over MCP it is done, #94).
- `CHK-101`, `CHK-103` and `CHK-105` as listed above, and the matching `get_architecture` conventions (#45).
