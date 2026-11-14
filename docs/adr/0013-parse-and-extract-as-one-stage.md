# 0013 – Parse and extract form one pipeline stage

Status: proposed

## Context

Blueprint 7 lists Parse and Extract symbols as two stages. The application layer (WP-19) must not know Tree-sitter types (blueprint 4.1), so a syntax tree cannot cross a port. Keeping all trees in memory between the two stages would also endanger the memory budget of 500 MB per 1 million LOC (blueprint 16.1).

## Options

1. Two stages, with an opaque tree type passed between them.
2. One stage `Analyze`: the adapter parses one file and extracts its symbols, the tree stays inside the adapter.

## Decision

Option 2. The pipeline has six stages: Fetch, Discover, Analyze, Resolve, Compute metrics, Persist. The scan view may later show Parse and Extract as sub-steps of Analyze through the progress event, without a new stage.

## Consequences

- Blueprint section 7 (diagram and table) is changed accordingly.
- Memory stays proportional to one file per worker, not to the whole project.
- A resolver or metric that needs syntax trees of other files is not possible; none is planned, because the global view comes from the symbol table in Resolve.
