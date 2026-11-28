# Acceptance: ABAP through abapGit folders (ADR 0113)

State of 2026-11-28, `main` with packages 1 to 3b merged (#820, #823, #830, #842). Verdict: **met for the basic tier**; the open points are listed at the end and have issues.

| Criterion | Verdict | Evidence |
| --- | --- | --- |
| The ABAP pack reads classes, interfaces, methods, functions and forms; names are folded to lower case | met | golden files and tests of the pack (`crates/adapters/parsing/tests/abap.rs`), highlight test |
| The pack is checked against a public repository | met | corpus `corpus/abap/corpus.toml` (abap2xlsx, pinned commit: 103 files, 101 classes, 11 interfaces, 1155 methods, 4 forms), counts checked against an independent count of the syntax tree. The source is fetched for the test only and never checked in or shipped (internal use) |
| An abapGit repository is recognised and its objects counted by type | met | `codeskimr_application::abapgit` (8 tests), `scope.get` field `abapgit`, console line of `scope` (tests in `ui_commands/scope_tests.rs`) |
| The project page shows what the repository holds | met | card on the tab Scope (`AbapGitCard`, two tests) |
| The explorer shows package, class and method | met | `explorer_tests.rs` (packages, programs on the package, files in the project folder), `outline` test of the store, pack test for the pack key `namespace` |
| Syntax highlighting of `.abap` files | met by tests only | `languages/abap/highlights.scm` with the highlight test of the parsing crate; the source column was not looked at with an ABAP project in this run |

Known limits (also in the ADR): no visibility; complexity counts decision keywords; references only by name (`PERFORM`, `CALL METHOD`, `name( )`), not `CALL FUNCTION 'X'`, `SUBMIT`, `INCLUDE`; three files of the corpus have grammar errors (`zcl_excel`, `zcl_excel_ranges`, `zcl_excel_worksheet`, at a variable called `ranges`); the package is the path of the folder; a Git project whose `STARTING_FOLDER` is not `src` is not recognised.

Not done in this run: the screens were not clicked through by hand with a real SAP project (the end-to-end test `web/e2e/ui-abapgit.spec.ts` drives the real server with a small abapGit folder written for the test and writes the screenshots `docs/screenshots/abapgit/` in all three themes: window "New project" with its hint, card on the tab "Scope", package tree of the explorer). Mock 27 (`docs/mocks/AbapGit.html`) was drawn after the screens, from them; the original draft is not in the repository. Hint in the window "New project" (issue #843): built (`scope.inspect`, `AbapGitHint`), but only a repository with `.abapgit.xml` gets it. Open: #845 and #846 (package name relative to the starting folder, package names in the graph, layers and rules).
