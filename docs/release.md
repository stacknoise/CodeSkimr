# Release process (preview and later releases)

1. Set `version` in the root `Cargo.toml` (for example `0.9.0-preview.1`; `Cargo.lock` follows with `cargo update --workspace`) and update `docs/release/preview-de.md` and `preview-en.md` (the German file is the text of the release).
2. Merge to `main` with green checks.
3. Actions, **Release**, Run workflow, input: the same version without `v`. The workflow checks that the version matches `Cargo.toml`, builds the UI (`web/`, embedded into the program by `crates/app/build.rs`) and the program on Linux and Windows, packs them with `tools/release/package.sh` and attaches `.zip`/`.tar.gz` plus `.sha256` to a **draft pre-release** `v<version>`.
4. The product owner tries the packages and publishes the draft (this creates the tag). Nothing is public before that.

macOS is not part of the preview (issue #131). The package contains the program, `languages/`, `default.toml`, the manual, the licence and the two preview notes.
