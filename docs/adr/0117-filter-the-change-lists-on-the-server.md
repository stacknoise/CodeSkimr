# 0117 – Filter the change lists on the server

Status: accepted (package of the coordinator, 2026-11-28; issue #826; follow-up of ADR 0097 and ADR 0092)

## Context

The comparison of two scans (ADR 0097) cuts every list of symbols to `compare.max_symbols` (200 by default). The screen "Change" filtered in the browser, on the symbols the answer held. With a large difference (2,305 symbols in one real case) a search saw only the first 200 of each list, and a symbol beyond the cut could not be found, although the screen says that the total is larger. The console had no filter at all and pointed to `--json` with `jq`.

## Options

1. **Filter on the server, before the cut.** The request carries the filter; the lists are filtered first and cut afterwards, so every hit is found and the totals are those of the hits.
2. **Raise `max_symbols`.** The answer grows with the difference and still ends somewhere.
3. **Keep the filter in the browser and load the whole difference.** A very large answer for a screen that shows a page of it.

## Decision

Option 1.

1. **The filter.** `ChangeFilter` (domain, `compare_filter.rs`) has a name (the text a qualified name contains, ignoring case), a cause (`added`, `changed`, `removed` or `moved`), a kind of symbol, `public_only` and `only_open`. A field left empty lets everything pass. `Comparison::filtered` keeps what passes, in the same order. The name and the kind are those of the symbol in the target scan for `changed` and `moved`, and of the base scan for `removed`; `public_only` is judged on the same side.
2. **Before the cut.** `ScanComparer::compare` filters the difference first. The lists, their totals and `is_cut` describe the symbols that passed. The layer changes and the critical symbols are not filtered: they are facts about the whole difference.
3. **Adapted is judged on the whole difference.** A usage counts as adapted when the symbol around it is touched by the difference (ADR 0097, decision 4). The filter does not change that: the usages of a symbol that passes still show as adapted or not touched as without the filter.
4. **`only_open`** keeps in the impact only the symbols with at least one place that no change touches. It needs the usages of every symbol that passes the other criteria, so with `only_open` every such symbol is read before the cut and the total counts the symbols that stay. Without it only the symbols that are listed are read, as before. It does not change the lists of the symbols themselves and counts only where there are usages (`scans.impact`, `change`, `impact_of_diff`).
5. **One filter for every surface** (ADR 0092). `scans.compare` and `scans.impact` take the optional inputs `name`, `cause`, `kind`, `public_only` and `only_open`; `codeskimr compare` and `codeskimr change` take `--match TEXT`, `--cause CAUSE`, `--kind KIND`, `--public` and `--only-open`; the MCP tool `impact_of_diff` takes the same five. An unknown cause or kind is `API-101`.
6. **The screen** sends the filter with the comparison, a short pause after the last change (a search is not sent for every letter), and shows the answer. It keeps its own filter of the rows as before, so what the server returns and what the screen shows cannot differ. The kinds of symbol to choose from are those that any answer of the comparison had, so choosing a kind does not shrink the choice. The filter row stays when the server finds nothing.

## Consequences

- A search finds every hit, however large the difference. The figures on the screen (symbols, usages, open places) describe what passed the filter.
- `only_open` on a very large difference reads the usages of every symbol that passes; it is bounded by the number of symbols and by `compare.max_usages` per symbol, and it only runs when the filter is on.
- A change of the filter on the screen is one more call of `scans.impact`; the answer of the earlier call is dropped when a newer one is on its way.
- No new setting, error code or schema version: the filter is part of the request.
