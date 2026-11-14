# 0046 – Error code catalogue catch-up (appendix A)

Status: accepted

## Context

Appendix A of the blueprint lists the error codes of `errors/error-codes.toml`. Since its last update the codes `PRJ-501`, `API-502` and `API-503` were registered in the file but not added to the appendix (issue #46; `MCP-202`, `MCP-203`, `CFG-103`, `STO-601` and `STO-901` were already added).

## Decision

1. Appendix A gets the three missing rows with the developer texts of the file.
2. The appendix stays a snapshot; `docs/error-codes.md`, generated from the file, is the current catalogue.

## Consequences

- No behaviour changes; the blueprint says what the code already registers.
