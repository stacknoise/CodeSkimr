# 0041 – A denied command and an unknown command answer alike

Status: proposed

## Context

Blueprint 10.1 says a denied command answers `AUTH-401`. The server checks existence first (`API-201` for an unknown id) and the permission second (`AUTH-401`). A caller without the right to run a command can therefore tell the commands that exist from those that do not (issue #269, review of #88). Blueprint S-06 asks for the team edition (stage T1) that denial and "not found" are compared for identical responses. In 1.0 the single local owner holds every permission, so nothing is revealed today; the limited principals of the MCP and remote paths are the first to see the difference.

## Options

1. Keep the order: unknown is `API-201`, denied is `AUTH-401`. Clear messages, but the existence of a command is an oracle for a caller without the right.
2. A caller who may not run a command gets the same answer as for an unknown id (`API-201`, status 404). `AUTH-401` stays for a missing or wrong installation token and for the permission check of routes that are not commands. The command list already shows only the commands the caller may run, so nothing contradicts it.
3. Both cases answer `AUTH-401`. Hides existence too, but turns every typo into "denied" and breaks the 404 contract of the REST description.

## Recommendation

Option 2. It meets S-06 with one change in the order of `routes::run` and the MCP tool lookup, keeps typos readable, and makes the team edition a matter of grants, not of new code paths.

## Consequences if accepted

- Blueprint 10.1 and S-06 name option 2; `docs/server.md` and `docs/error-codes.md` follow.
- `routes::run`: the permission check moves in front of the existence answer; a test with a restricted principal asserts identical status and body for an unknown and a denied command.
- A user whose rights are too low is no longer told so by the API; the UI must explain missing rights from the command list.
