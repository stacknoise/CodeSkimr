# 0012 – Agents may merge after review

Status: proposed

## Context

Blueprint 18.2 says a PR is merged only when CI is green and the product owner has approved. In practice the product owner merges every PR by hand, which makes him the bottleneck while several work packages run in parallel. The ruleset already requires 0 approvals and the required status checks protect `main`.

## Options

1. Keep manual merging by the product owner for every PR.
2. Let agents merge after green required checks and a reviewer OK, with defined exceptions that stay with the product owner.
3. Let agents merge without a separate review.

## Decision

Option 2.

- Merge happens by auto-merge (squash), enabled by the coordinator agent, only when all required checks are green and the reviewer agent has written an OK as a review comment.
- The author agent never merges its own PR without that review.
- The product owner merges these himself: PRs labelled `security-sensitive` or `new-dependency`, and every change to `BLUEPRINT.md`, `docs/adr/`, `CLAUDE.md`, `.claude/agents/` and to repository rules.
- Agents still never change repository settings or rulesets.

## Consequences

- The repository setting "Allow auto-merge" must be enabled once by the product owner.
- Rules in `CLAUDE.md`, `.claude/agents/` and the planning document are updated accordingly.
- Quality gates stay the same: required checks plus an independent review.
