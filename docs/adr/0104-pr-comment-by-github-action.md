# 0104 – Comment with the impact of a change in the pull request, by a GitHub Action

Status: proposed (way A of concept E2 decided by the product owner on 2026-11-23; builds on ADR 0097)

## Context

Concept E2 wants one comment in a pull request that sums up the view "Change" (ADR 0097): changed public symbols, the usages that the pull request did not touch, new layer violations and cycles. The comment is updated on every push, never duplicated. The product owner decided way A: a ready-made GitHub Action writes the comment with the token GitHub gives the workflow. CodeSkimr itself does not talk to GitHub and stores no token (way B was rejected).

ADR 0097 compares two retained snapshots of one project and leaves a Git branch as base out. A fresh CI checkout has no snapshots, so the Action has to produce them.

## Options

1. A Git base inside CodeSkimr (`--base main`, scan of a temporary snapshot). Own ADR, touches retention, clone and credentials.
2. The Action scans the base state, then the pull request state (incremental), and asks for the comparison of the two newest snapshots. No new CodeSkimr feature besides the Markdown output.

## Decision

Option 2.

1. **Markdown output.** `codeskimr change <project> [--base <id>] [--target <id>] --markdown` prints the answer of ADR 0097 as Markdown (same use case, same data as `--json`; delivered with the CLI package of ADR 0097). The first line is the fixed marker `<!-- codeskimr-change -->`. The text says that only direct usages are known and says when the answer was cut. Texts in German and English (Fluent), language by `--lang` or the configuration.
2. **Action.** A composite Action in `actions/change-comment/` of this repository. Steps: download the release program for the runner and verify its checksum; check out the base commit of the pull request and scan it; check out the head commit and scan again; run `change --markdown`; write the comment. The data folder lives in the runner's temporary directory.
3. **Comment.** The script finds an earlier comment by the marker and updates it, otherwise creates one (GitHub REST API through `gh api`, token from the `github-token` input, default `github.token`). An empty change (no changed symbols) updates the comment to say so. Needs `pull-requests: write`.
4. **Forks.** For pull requests from forks the token is read-only; the Action then writes the Markdown to the job summary and does not fail. Using `pull_request_target` is not recommended and the documentation says why.
5. **Check (way C).** Input `fail-on` (default `none`): `new-layer-violation` or `new-cycle` fails the step when the comparison reports one. Never on by default.
6. **Other CI systems** use the Markdown from `codeskimr change --markdown` the same way; the Action is only the GitHub glue.
7. **No new dependency** in the Rust code, no token in CodeSkimr, no network access besides the download and the GitHub API call of the Action.

## Consequences

- The user adds one workflow file. The documentation shows it for de and en.
- The first scan of a large project in CI is a full scan; the cache of the data folder (`actions/cache`) is an optional input, not part of this decision.
- A shallow checkout is not enough: the Action fetches both commits itself.
- Level 2 stays off in the Action (the comparison uses level 1 symbols only, ADR 0097).
