# Hand test of 1.0.0-rc.6 on Windows (M1 to M6)

Run by the product owner on 2026-11-24 on his Windows laptop with the downloaded program file of the release draft `1.0.0-rc.6` (no checkout, no cargo). The task list covered only what the CI cannot show: the private C# reference (ADR 0017), a private Git repository, the dialogs of the system and the look of the screens. Results were reported in the project chat. The detailed list of the usage sample was judged by Claude against the source of the private project; no name, path or absolute count of that project is recorded here, only rates and verdicts (ADR 0017, ADR 0011).

This page adds to the reports `m1-m2.md`, `m3.md`, `m4.md`, `m5.md` and `m6.md`; it does not replace them. Acceptance itself stays the product owner's decision.

Legend: ✅ passed · ◐ passed with a named limit or finding · ⏭ voluntary, not run

## Results per task

| Task | Milestone | Result | Evidence |
|---|---|---|---|
| Preparation: checksum of the download, version, `--help` | all | ✅ | Reported as passed. |
| Private reference by script: counts, scan time, incremental scan, MCP tools | M1, M2 | ✅ | Three scans in fresh databases give the same counts. Full scan: over the budget of 1 s per 100k LOC, with and without the tests, the known limit of #410. Incremental scan after one appended line: within the budget of 300 ms. `tools/list` shows 13 tools; every query is fast. |
| An agent uses the MCP server | M2 | ✅ | Claude Code in the Claude desktop app searched symbols and listed the usages of a method through `codeskimr mcp --project`. See the notes for how the server had to be registered. |
| Usage sample of 20 symbols | M2 criterion 4 | ◐ | Two runs with rc.6 on different symbols, see the next section: every usage of the source was found in both runs, every hit marked `certain` is correct, wrong hits are all `uncertain`; finding #685. Repeat run with rc.7: see the repeat run below (19 of 20 symbols complete, #685 fixed, new finding #705). |
| Large public C# project for time | M1 | ⏭ | Not run; the limit is decided (#410). |
| Start, tray and start file | M3, M6 | ✅ | `open --no-browser` prints the start file; `icacls` shows only the user on the file and on its folder; the file is gone after its time; "Quit" in the tray leaves no process. Covers the Windows side of #104 and #659. |
| Folder dialog | M3, M6 | ◐ | The dialog opens in front, a chosen folder is scanned, a second click while the dialog is open is refused. After that refused click a third click opened a second dialog: #692, fixed with #693 and confirmed with rc.7, see the repeat run below. |
| Screens, keyboard, themes, language | M3 | ✅ | Reported as "looks good". |
| Map, scan and usages on the private project | M4 | ✅ | F8 on the usages screen and Alt+click in the source pane work. |
| Large project (VS Code checkout) while a console scan runs | M4 | ◐ | The UI stays usable during the scan, the statistics page loads in a few seconds, and the small project is fast afterwards. Findings: #695 (the projects list shows the project as never scanned while the console scans it; correct again after the scan) and #694 (the UI does not show a scan of the console at all). The full scan of this public project took 344 s for 16,717 files, of which the screen "Scans and performance" attributes 334 s to parsing TypeScript; reported to the scan-speed work, not a criterion of M4. |
| Sign "approximation" in the UI | M5 | ✅ | The accuracy card of the usages screen carries the chip for C# and says that level 2 is not confirmed for the project and that all hits are unverified. The scan output names each missing level 2 tool with its purpose and an install hint. |
| Level 2 on Windows | M5 | ⏭ | Not run. |
| Stored login (`credential set`) | M6 | ✅ | Stored; the secret is not printed. |
| Private Git repository by script | M6 criterion 6 | ✅ | Scan with the stored login, update, export: exit 0 each. The secret occurs in none of the three command outputs at trace level, not in the export page, in none of the files of the data folder and in none of the files of the Git cache; no Git configuration with credentials in an address; no token pattern. Without a login, in a fresh data folder and with the credential helpers of the system switched off, the scan fails with `SRC-401` and does not ask. With the helper of the system it succeeds, as blueprint 12 intends. |
| HTML export from the UI | M6 | ✅ | Reported as working: the saved page shows its screens with content when opened from disk (with rc.5 they stayed empty on Windows). |

## Usage sample (M2 criterion 4)

Both runs asked `find_usages` with `include_tests = true` for 20 symbols picked by the script. The hits were compared with the source.

| Run | Symbols complete | Symbols without a wrong hit | Recall | Precision |
|---|---|---|---|---|
| `src` only (tests not scanned) | 20 of 20 | 16 of 20 | 100 % | 76 % |
| Whole repository with tests | 20 of 20 | 15 of 20 | 100 % | 68 % |

- Every hit marked `certain` is correct. Every wrong hit is `uncertain`.
- One cause for all wrong hits of the first run: a member of the same name on another type when the type of the receiver is not known (lambda parameter, `foreach` variable, result of a call, index access). This belongs to #134 and #127.
- In the second run one project method with a very common name was given many calls of a method of the runtime library with the same name (#685; the product owner decided to fix it before 1.0). Without that symbol the precision is 85 %. One symbol shows that overloads are not told apart.
- Calls through an interface to its only implementation were counted as correct, as in the earlier samples.

Proposed verdict: met with the named approximation. Completeness, which the product owner ranks first, is 100 % in both runs; the precision is that of level 1 for C#, which is shown as an approximation in the UI and in the release notes (decision of 2026-11-20).

## Findings

| Issue | What | Decision |
|---|---|---|
| #685 | C#, level 1: calls of a library method listed as usages of a project method of the same name | before 1.0; fixed with #689 and confirmed with rc.7 |
| #705 | C#, level 1: a call inside `if (!await field.Method())` is not listed as a usage (the one missing usage of the rc.7 sample) | not a blocker for 1.0 |
| #692 | Folder dialog: a refused click released the lock, the next click opened a second dialog | fixed with #693, confirmed with rc.7 |
| #695 | Projects list shows a scanned project as never scanned while the console scans it | before 1.0 |
| #694 | The UI does not show scans started in the console (state and progress) | after 1.0, with the job register of the panel "background processes" |

## Notes on the scripts and the manual

- `private-reference.ps1` asked without `include_tests` until this change (the fault of the run of 2026-11-21) and left `CODESKIMR_DATA_DIR` set in the caller's window; both are corrected.
- `m6-private.ps1` ran its last step ("without a login the scan must fail") in the data folder of the earlier steps. A project remembers the name of its login (never the secret), so that scan logged in again and the step reported a failure that was none. The step now uses a fresh data folder. A stale export page of an earlier run is removed before the export.
- The variants used in this hand test take the program file by a parameter instead of building it; they are kept outside the repository.
- The product owner has no `claude` command line; the manual describes only `claude mcp add`. In the desktop app the server had to be entered under `mcpServers` in the user's `.claude.json`. The manual should describe that way too.

## Repeat run with 1.0.0-rc.7 (2026-11-24)

Run by the product owner on Windows with the same scripts and the program file of rc.7. Rates and verdicts only (ADR 0017).

| Check | Result |
|---|---|
| Folder dialog, several clicks (#692) | ✅ No second dialog opens. |
| Version display, error code list, display of the effective settings in the UI (ADR 0092) | ✅ Reported as fine. |
| Counts of the three scans | Equal to each other. They differ from the baseline of the rc.6 run, as expected: the counted references changed between the two versions (#689). The run of rc.7 is the new baseline. |
| Full scan | Over the budget of 1 s per 100k LOC as before, the known limit of #410. |
| Incremental scan | Within the budget of 300 ms. |
| MCP tools | 13, complete; every query is fast. |

### Usage sample of rc.7 (M2 criterion 4)

20 symbols, queried with `include_tests = true`, judged against the source.

| Symbols complete | Symbols without a wrong hit | Recall | Precision |
|---|---|---|---|
| 19 of 20 | 17 of 20 | about 98 % | about 86 % |

- Every hit marked `certain` is correct; every wrong hit is `uncertain`.
- #685 is fixed: calls of a library method are no longer listed as usages of a project method of the same name.
- The wrong hits belong to known classes: a member of the same name on another type when the receiver type is not known (a lambda parameter or the result of a call), `List.IndexOf` of a local list, and overloads that are not told apart.
- One real usage is missing: a call of the form `if (!await field.Method())` in production code, while the mock set-ups of the same method in tests are found (#705, no blocker for 1.0).

Proposed verdict for M2 criterion 4: met with the named approximation. Completeness is about 98 % (19 of 20 symbols complete; the 100 % of the rc.6 runs was not reached this time because of the one missing usage of #705), and the rate of correct hits rose from 68 % (rc.6) to about 86 %.
