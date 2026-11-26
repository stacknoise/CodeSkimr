# Comment in the pull request

A GitHub Action writes one comment into every pull request: which symbols the change touched and which usages it left untouched. The comment is updated on every push; no second one appears. CodeSkimr stores no token and does not talk to GitHub; the Action uses the token GitHub gives the workflow.

```yaml
name: CodeSkimr
on: pull_request
permissions:
  contents: read
  pull-requests: write
jobs:
  change:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: stacknoise/CodeSkimr/actions/change-comment@v1.1.0
        with:
          version: 1.1.0
          language: en
```

The Action scans the base commit, then the head commit of the pull request, and posts `codeskimr change --markdown` (the two newest scans). It runs on Linux runners. Inputs: `version` (release of CodeSkimr), `language` (`de` or `en`), `fail-on` (`none`, `new-layer-violation` or `new-cycle`; default `none`), `github-token`.

- **Forks.** The token of a pull request from a fork cannot write comments. The Action then writes the text to the job summary. Do not switch to `pull_request_target` for this: it would run code of the fork with write rights.
- **Limits.** Only direct usages are known; reflection, strings and other repositories are not seen. The comment says so.
- **Other CI systems** call `codeskimr scan .` twice (base, head) and `codeskimr change . --markdown` and post the text themselves.
