# CodeSkimr: rules for all agents

CodeSkimr scans code repositories and turns them into navigable documentation, with a browser UI, a CLI, a REST API and an MCP server. Rust core, SolidJS frontend.

The binding specification is `BLUEPRINT.md` (added by work package WP-06; until it is merged, ask the coordinator for the current text). Every rule in it applies to every pull request. If a rule is wrong, change the blueprint through an ADR first, never work around it in code.

## Language

- Code, identifiers, comments, commit messages and technical docs: English.
- User-visible texts: German and English, through the Fluent files in `locales/`.
- Pull request descriptions and chat replies to the product owner: German, in plain language that can be reviewed without Rust knowledge.

## Working rules

1. One agent, one work package, one branch, one PR. Touch only files that belong to the package (see the `area:` labels).
2. Read the blueprint sections named in the issue before writing code.
3. Branches: agent sessions work on `claude/...` branches. The PR title uses Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`); the PR is squash-merged, so the title becomes the commit.
4. From milestone M1 on, use CodeSkimr's own MCP server: `find_similar_code` and `find_constants` before writing new code, `check_conventions` before every commit.
5. Never merge, never change repository settings or rulesets, never loosen a lint, threshold or budget, never disable or skip a test to get CI green.
6. The ruleset requires 0 approvals because PRs are opened under the product owner's account. The reviewer agent writes its verdict as a review comment, never for its own work. Only the product owner (Anton) merges.
7. A change to the blueprint, a new dependency layer, symbol kind, pattern or technology needs an ADR in `docs/adr/` and the label `blueprint-change`.
8. A new dependency needs one sentence of justification in the PR and the label `new-dependency`.
9. Never commit secrets, credentials, tokens, private paths or data from private projects. This repository is public.
10. If something is unclear or blocked: set `status:blocked` and ask the coordinator. Do not guess.

## Binding code rules (short form, see blueprint section 3)

No duplication, functions of at most 40 lines and cognitive complexity 15, no literals in production code (values live in `consts`, `config/default.toml`, `locales/`, `errors/error-codes.toml`, `languages/`, `design/tokens.toml`), no panics in production code, no `unsafe` outside `codeskimr_secrets::memory`, every public item documented, dependency direction as in blueprint 4.2.

## Definition of done

Blueprint 18.4: rules met and CI green, tests on the right level, texts in de and en, new errors registered, budgets kept, UI matches the mock, docs updated.

## Roles

Role descriptions are in `.claude/agents/`. Workflow: coordinator releases a package, developer implements and opens the PR, CI runs, reviewer checks and comments (security reviews `security-sensitive` PRs), Anton merges.
