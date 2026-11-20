# Changelog

All notable changes of CodeSkimr. The format follows [Keep a Changelog](https://keepachangelog.com/), the versions follow [SemVer](https://semver.org/).
Each section is generated from the Conventional Commit titles of the squash-merged pull requests with `tools/release/changelog.py` (see `docs/release.md`); the titles are English, as CLAUDE.md requires for commit messages (`changelog.py` refuses German titles).

## [Unreleased] - 2026-11-15

### Added

- "Browse" button with the folder dialog of the system (Windows and macOS) next to the folder field of a new project (ADR 0075)
- remove a project from the app again: command `projects.remove` and a "Remove" button with a confirmation in the project list
- coupling metrics Ce, Ca and instability per snapshot ([#275](https://github.com/stacknoise/CodeSkimr/pull/275))
- `#private` members in TypeScript and JavaScript are private (ADR 0047, #81) ([#323](https://github.com/stacknoise/CodeSkimr/pull/323))
- tool check reports an SDK that is newer than global.json allows ([#328](https://github.com/stacknoise/CodeSkimr/pull/328))
- refuse a junction as the launch file folder on Windows ([#370](https://github.com/stacknoise/CodeSkimr/pull/370))
- tie a command request to its command type ([#362](https://github.com/stacknoise/CodeSkimr/pull/362))
- seal principal construction and command execution by types ([#343](https://github.com/stacknoise/CodeSkimr/pull/343))
- embed the language packs in the program (#327) ([#350](https://github.com/stacknoise/CodeSkimr/pull/350))
- check macro_rules bodies where they are defined (no_literals) ([#346](https://github.com/stacknoise/CodeSkimr/pull/346))

### Fixed

- level 1 usages stay visible where level 2 finds nothing, marked "unchecked (level 1)"; verified ones read "verified (level 2)" (ADR 0033 addendum)
- link check when opening and a hardened answer buffer ([#322](https://github.com/stacknoise/CodeSkimr/pull/322))
- error text for unreadable sources, storage error tests and stale Git work folders ([#319](https://github.com/stacknoise/CodeSkimr/pull/319))
- finish the follow-ups of the embedded packs: directory sync, open without packs, pack check beside the program (#351) ([#383](https://github.com/stacknoise/CodeSkimr/pull/383))
- capture more Rust locals so free names stop matching functions ([#385](https://github.com/stacknoise/CodeSkimr/pull/385))
- let the Rust pack treat bare names in functions as no members, type the struct shorthand (#81) ([#377](https://github.com/stacknoise/CodeSkimr/pull/377))
- let the UI and the MCP service share one store ([#380](https://github.com/stacknoise/CodeSkimr/pull/380))
- Java level 2 reads implicit constructor calls and import lines from the text (`cd0475d`)
- a languages folder in the current folder no longer replaces the embedded packs, remove old pack folders (#351) ([#369](https://github.com/stacknoise/CodeSkimr/pull/369))
- order usages verified first before the limit cuts them ([#367](https://github.com/stacknoise/CodeSkimr/pull/367))
- stop a bare name in a Python function from matching a field, measure Java, Python and C# certain precision (#81) ([#355](https://github.com/stacknoise/CodeSkimr/pull/355))
- clean up stale pack staging folders, sync written packs, skip packs for credential commands (#351) ([#359](https://github.com/stacknoise/CodeSkimr/pull/359))
- build the usages index ahead of the first query and after writes ([#356](https://github.com/stacknoise/CodeSkimr/pull/356))
- tool check reads global.json like the SDK and all manifest folders (#207, #208) (`ffd20c8`)
- usages legend and name wrap, settings note, frame test per theme, ADR 0052 for Rust names (#331, #332, #335, #336) ([#352](https://github.com/stacknoise/CodeSkimr/pull/352))
- SCIP import no longer quadratic on very large or single-line files ([#220](https://github.com/stacknoise/CodeSkimr/pull/220))
- recalculate coupling after the level 2 import ([#342](https://github.com/stacknoise/CodeSkimr/pull/342))
- remove parked Git copies left by a crash during the swap ([#349](https://github.com/stacknoise/CodeSkimr/pull/349))
- measure TypeScript level 1 accuracy and stop a named receiver from matching a local function as certain (#81) ([#345](https://github.com/stacknoise/CodeSkimr/pull/345))
- close command oracle, send budget for event streams, read_body error class (#269, #273, #301) ([#315](https://github.com/stacknoise/CodeSkimr/pull/315))
- level 2 indexer hardening, consent and content hash (#230, #231) (`f2c0aa9`)

### Changed

- split remaining long web components and enforce max-lines-per-function ([#371](https://github.com/stacknoise/CodeSkimr/pull/371))
- split long web components and add golden cases for owner lookup limits ([#360](https://github.com/stacknoise/CodeSkimr/pull/360))

### Documentation

- changelog generator, README and checklist for the 1.0 release (WP-81, WP-83) ([#326](https://github.com/stacknoise/CodeSkimr/pull/326))
- result of the sample of 20 usages in the acceptance report (WP-38) ([#312](https://github.com/stacknoise/CodeSkimr/pull/312))
- report uses of symbols that level 1 does not have (`2b9e927`)
- repeat the M3 and M4 acceptance runs ([#338](https://github.com/stacknoise/CodeSkimr/pull/338))

### Tests

- pin the literal rules of the frontend ([#341](https://github.com/stacknoise/CodeSkimr/pull/341))
- cover secret clearing of the credential form on cleanup and send failure ([#382](https://github.com/stacknoise/CodeSkimr/pull/382))
- credential store against the real OS keystore in CI (#197, #320) ([#353](https://github.com/stacknoise/CodeSkimr/pull/353))
- benchmark scans of a Git source ([#354](https://github.com/stacknoise/CodeSkimr/pull/354))

### Maintenance

- adapt the lock file of import_scip to new dependencies ([#329](https://github.com/stacknoise/CodeSkimr/pull/329))
- fail CI on tracked paths that differ only in letter case ([#340](https://github.com/stacknoise/CodeSkimr/pull/340))

## [0.9.0-preview.1] - 2026-11-14

### Added

- Make budgets measurable: CPU time, quick-search benchmark, limit for frequent fingerprints (#141, #23, #144, #99) ([#294](https://github.com/stacknoise/CodeSkimr/pull/294))
- Store theme and language on the server, editor link encodes special characters ([#316](https://github.com/stacknoise/CodeSkimr/pull/316))
- find_constants also reports configuration and translation keys (#100) ([#308](https://github.com/stacknoise/CodeSkimr/pull/308))
- Create credentials in the UI and choose them for a new project (WP-76, #161) ([#314](https://github.com/stacknoise/CodeSkimr/pull/314))
- Finish the embedded app (export, start test, Windows permissions, hardening) ([#292](https://github.com/stacknoise/CodeSkimr/pull/292))
- GitLab token refresh for Git credentials (WP-76, #161) ([#295](https://github.com/stacknoise/CodeSkimr/pull/295))
- Rust impl blocks belong to their type (containers without a symbol, ADR 0039) ([#288](https://github.com/stacknoise/CodeSkimr/pull/288))
- parse time per language in the scan screen ([#299](https://github.com/stacknoise/CodeSkimr/pull/299))
- check_conventions compares duplicates with the stored project state (#94) ([#287](https://github.com/stacknoise/CodeSkimr/pull/287))
- Close the remaining M3 UI deviations (#171, #178, #193, #158, #159, #140) ([#281](https://github.com/stacknoise/CodeSkimr/pull/281))
- Constant signature with the full declaration (Java, C#), length limit for signatures ([#280](https://github.com/stacknoise/CodeSkimr/pull/280))
- Release workflow for preview versions (Windows/Linux packages) ([#267](https://github.com/stacknoise/CodeSkimr/pull/267))
- Level 2 measurement for Java and C#, constructor calls as type usages, overload heads only without a symbol of their own ([#260](https://github.com/stacknoise/CodeSkimr/pull/260))
- qualified names of namespaced languages without the file ([#254](https://github.com/stacknoise/CodeSkimr/pull/254))
- Show the tool check when a project is loaded ([#206](https://github.com/stacknoise/CodeSkimr/pull/206))
- Confirm and revoke level 2 from the CLI, blueprint and ADR 0033 updated (B2) ([#235](https://github.com/stacknoise/CodeSkimr/pull/235))
- Accuracy measurement for Rust (rust-analyzer) and Java (javac) ([#236](https://github.com/stacknoise/CodeSkimr/pull/236))
- "Types" level and type focus of the dependency map ([#233](https://github.com/stacknoise/CodeSkimr/pull/233))
- Second pass in the scan that takes level 2 references from SCIP indexes (B1) ([#228](https://github.com/stacknoise/CodeSkimr/pull/228))
- Code line for each hit in the result list (#157), room under the graph for the legend ([#227](https://github.com/stacknoise/CodeSkimr/pull/227))
- Start indexers for level 2 references (runner, consent, permissions, limits) ([#221](https://github.com/stacknoise/CodeSkimr/pull/221))
- Depth steps of the namespace level on the dependency map ([#215](https://github.com/stacknoise/CodeSkimr/pull/215))
- SCIP import for level 2 references (decoder, mapping, replace per file) (`5d192f1`)
- Language table with tier and symbols, branch/commit and server-side stage durations on the scan screen ([#214](https://github.com/stacknoise/CodeSkimr/pull/214))
- Number of index entries in the footer of the quick search (part of #178) ([#212](https://github.com/stacknoise/CodeSkimr/pull/212))
- Cancel a scan (button and Esc) and core count on the scan screen ([#211](https://github.com/stacknoise/CodeSkimr/pull/211))
- Documentation and metrics in the quick-search preview (part of #178) ([#205](https://github.com/stacknoise/CodeSkimr/pull/205))
- Show the level of a symbol with G and Ctrl+Enter in the dependency map (part of #171, #178) ([#203](https://github.com/stacknoise/CodeSkimr/pull/203))
- Performance screen shows measured times, status bar shows the query time ([#204](https://github.com/stacknoise/CodeSkimr/pull/204))
- Layer bands and namespace level in the dependency map ([#199](https://github.com/stacknoise/CodeSkimr/pull/199))
- Open a symbol in the editor with E and Alt+Enter (part of #171) ([#200](https://github.com/stacknoise/CodeSkimr/pull/200))
- Manage credentials through the server and use them with Git projects (WP-76, #161, #184) ([#196](https://github.com/stacknoise/CodeSkimr/pull/196))

### Fixed

- Close gaps in the C# and TypeScript language packs (#77, #183, #81) ([#317](https://github.com/stacknoise/CodeSkimr/pull/317))
- Harden level 2 indexers: process and files (#222, #223, #229) (`a6bb5c2`)
- Close the last security findings for secrets and server (#51, #52, #68, #273, #269) ([#296](https://github.com/stacknoise/CodeSkimr/pull/296))
- Close security alerts (minimatch, CodeQL warning in secret_url) ([#305](https://github.com/stacknoise/CodeSkimr/pull/305))
- Record C# access through ?. as a usage ([#303](https://github.com/stacknoise/CodeSkimr/pull/303))
- retry the switch to the write-ahead log on a new database file ([#304](https://github.com/stacknoise/CodeSkimr/pull/304))
- Release run replaces a leftover draft ([#293](https://github.com/stacknoise/CodeSkimr/pull/293))
- scan limits for foreign projects, parallel scans and git inputs (#54, #125) ([#286](https://github.com/stacknoise/CodeSkimr/pull/286))
- UI builds on Windows (file names Preview and preview collided) ([#285](https://github.com/stacknoise/CodeSkimr/pull/285))
- C# resolver recognises more receiver forms and no longer spreads base calls ([#283](https://github.com/stacknoise/CodeSkimr/pull/283))
- Concurrent first opening of the database no longer fails on the migration ([#284](https://github.com/stacknoise/CodeSkimr/pull/284))
- MCP rescan shares the scan lock and id assignment with the UI (#91, #121, #155) ([#271](https://github.com/stacknoise/CodeSkimr/pull/271))
- server limits for event streams, connections and slow clients (#88) ([#268](https://github.com/stacknoise/CodeSkimr/pull/268))
- Watch mode ignores the data folder inside the watched folder ([#279](https://github.com/stacknoise/CodeSkimr/pull/279))
- MCP start loads language packs only once, e2e tree test waits for the cursor ([#270](https://github.com/stacknoise/CodeSkimr/pull/270))
- Creating a project assigns the id atomically, mcp creates nothing for never-scanned folders ([#277](https://github.com/stacknoise/CodeSkimr/pull/277))
- C# usages in the branches of a conditional expression (#263) ([#264](https://github.com/stacknoise/CodeSkimr/pull/264))
- Project list no longer fails when a stored tool report is unreadable ([#265](https://github.com/stacknoise/CodeSkimr/pull/265))
- Basic tier notices a broken pack, Ruby def in if/begin, C function-pointer return ([#266](https://github.com/stacknoise/CodeSkimr/pull/266))
- Level 2 maps arrow functions, getters/setters, local functions and @overload (#247, #248, #249) ([#256](https://github.com/stacknoise/CodeSkimr/pull/256))
- Level 2 reads Self (impl#[Type]) and trait methods from SCIP indexes (#240, #241) ([#245](https://github.com/stacknoise/CodeSkimr/pull/245))
- Raise the minimum version of scip-typescript to 0.3.11 (#209) ([#253](https://github.com/stacknoise/CodeSkimr/pull/253))
- Call scip-python with --project-version and without the ineffective project argument ([#251](https://github.com/stacknoise/CodeSkimr/pull/251))
- Tell apart two types with the same name on the type level of the map ([#238](https://github.com/stacknoise/CodeSkimr/pull/238))
- Legend, fixed graph height and "Calls / reads" column on the usages screen ([#213](https://github.com/stacknoise/CodeSkimr/pull/213))
- Level 1 keeps all hits and marks them as unchecked ([#186](https://github.com/stacknoise/CodeSkimr/pull/186))
- No double slash in the editor link (part of #171) ([#201](https://github.com/stacknoise/CodeSkimr/pull/201))

### Documentation

- Missing error codes in blueprint appendix A (#46) ([#318](https://github.com/stacknoise/CodeSkimr/pull/318))
- New run of the private reference in the M1 and M2 acceptance report (WP-28, WP-38) ([#302](https://github.com/stacknoise/CodeSkimr/pull/302))
- Bring the manual up to date with level 2 and the tool check (WP-81) ([#278](https://github.com/stacknoise/CodeSkimr/pull/278))
- Align the owner of the error code group RES in blueprint and registry ([#272](https://github.com/stacknoise/CodeSkimr/pull/272))
- Add HTML export to the manual (WP-81) ([#180](https://github.com/stacknoise/CodeSkimr/pull/180))
- Blueprint 16.1, 17.2 and appendix A with the measured level 2 numbers ([#262](https://github.com/stacknoise/CodeSkimr/pull/262))
- ADR 0036: names without file module for languages with namespaces (#137) ([#239](https://github.com/stacknoise/CodeSkimr/pull/239))
- M4 acceptance repeated (budgets and mock comparison) ([#226](https://github.com/stacknoise/CodeSkimr/pull/226))

### Tests

- Level 2 accuracy measurement for TypeScript and Python (spike) ([#252](https://github.com/stacknoise/CodeSkimr/pull/252))
- Level 2 accuracy measurement with a hand-made SCIP index ([#242](https://github.com/stacknoise/CodeSkimr/pull/242))

### Maintenance

- Release preparation (macOS as required check, lock file check, macOS package, trial run) ([#300](https://github.com/stacknoise/CodeSkimr/pull/300))
- forbid silencing no_literals and check the lint workspace in deps ([#313](https://github.com/stacknoise/CodeSkimr/pull/313))
