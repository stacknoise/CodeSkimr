# Checklist for the 1.0 release (milestone M7)

Blueprint 19, M7: installers or single executables for Windows, Linux and macOS, user docs, published error-code catalogue; acceptance: clean install on all three systems, all budgets green.
Only the product owner creates the tag and publishes the release (`docs/release.md`). Tick the boxes in the release pull request or the release issue.

## Before the release workflow

- [ ] Milestones M1 to M6 are accepted (reports in `docs/acceptance/`); open acceptance findings are issues with a decision.
- [ ] All budgets of blueprint 16 are green: `bench` is green on `main`, and the numbers in `docs/benchmarks.md` and the scan screen match the measurement.
- [ ] Accuracy per language is measured and stated in the release notes (`docs/accuracy.md`); languages below 99 % are named as approximations.
- [ ] No open issue with the label `security-sensitive` or `status:blocked`, except #289 and #290 (labelled `after-1.0`, deliberately postponed by the product owner on 2026-11-20); `cargo audit` and `cargo deny` are green (`deps`).
- [ ] The required checks in the ruleset match `docs/ci.md` (including `build (macos-latest)` and `test (windows-latest, indexer)`).
- [ ] `version` in the root `Cargo.toml` and `Cargo.lock` is the release version (no `-preview` suffix).
- [ ] `CHANGELOG.md` has the new section (`tools/release/changelog.py`), and breaking changes are under their own heading.
- [ ] Error-code catalogue is current: `docs/error-codes.md` is generated (`codegen` check is green); the manual chapter "Error codes" (`docs/manual/en/errors.md`, `docs/manual/de/fehler.md`) lists the codes users meet most.
- [x] Manual (de and en) is checked against the program: every command and option in the text exists in `codeskimr --help`, every link works, screenshots match the screens.
- [ ] The release notes are no longer the preview notes: `docs/release/notes-de.md` and `notes-en.md` (the text of the release page) are the notes for 1.0 (accuracy statement confirmed by the product owner, 2026-11-20).
- [ ] README and the release notes (`docs/release/`) say what is in this release and what is not (known limits, signing, installers).
- [ ] Single files exist for all three systems with `.sha256` and provenance attestation (decision on #327: unsigned, language packs embedded); signing is the first task after 1.0.
- [x] Product page (repository `stacknoise/homepage`) uses the same texts and version.

## Release workflow and clean-install probe

- [ ] Actions, Release, Run workflow with the version. The job `draft release` creates a draft; `package` builds the single file for Linux, Windows and macOS, its `.sha256` and the attestation; `clean install` checks the provenance and the sum, runs each file alone in an empty folder on a fresh runner and scans a small project.
- [ ] All jobs are green. If one is red, do not publish; fix, merge and run the workflow again (a leftover draft is replaced).
- [ ] By hand on real machines (the probe has no browser and no keystore): Windows, Linux with a Secret Service, macOS. Download the single file (on Linux and macOS `chmod +x`, on macOS remove the quarantine flag), start it, the browser opens, scan a project, open Explorer, Usages and the dependency map, export HTML, `codeskimr --version` shows the release version.
- [ ] Checksums of the downloaded files match the `.sha256` files, and `gh attestation verify <file> --repo stacknoise/CodeSkimr` succeeds for each.

## Publishing (product owner)

- [ ] Read the draft text (release notes, known limits). `release.yml` marks a draft as pre-release only for versions with a suffix (`1.0.0-rc.1`); the 1.0.0 draft is a normal release. Publish the draft; this creates the tag `v<version>`.
- [ ] Announce on the product page; link the release and the changelog.

## After the release

- [ ] Open the next `-dev` or preview version in `Cargo.toml` if work continues on `main`.
- [ ] Issues that came in during the release are triaged.
