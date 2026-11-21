# 0081 – Statistics screen: rankings over the symbols of a project

Status: accepted (Anton, 2026-11-21: layout chosen on the decision card, "in 1.0" the same day)

## Context

The explorer answers "what is this symbol", the usages screen "who uses it". Nothing answers the questions over the whole project: which symbols are used most, which are used by nobody, which functions are the most complex, the largest, the most coupled. All the numbers are in the database already (references, metrics of ADR 0074); only a view is missing. Blueprint 15 lists the screens, so a new one is a change of the blueprint.

A scan of CodeSkimr itself showed what such a view must not hide:

- About 92 % of the level 1 references are only "probable" (resolved by name). Sorted by all references, a common name such as `new` or `collect` wins although one usage is certain.
- "No usage found" is not "dead": level 1 reads the source text. Macros, reflection, entry points and users outside the project are not seen.
- Most symbols without a caller are test functions, which the test runner calls.

## Decision

1. A new screen **Statistics** in the navigation rail, after Dependencies. It is a project screen: its entry is shut until a project is open, like the explorer.
2. Layout as chosen by the product owner (mock `docs/mocks/Statistics.html`): the tab **Overview** with four numbers and four cards of five rows, and one tab per ranking: **Most used**, **No usage found**, **Complexity**, **Size**, **Coupling**. The complexity tab also draws a histogram of all functions and the "hotspots" (complexity against usages) of the rows of the ranking.
3. Every row leads into the explorer (arrow, double click, `Enter`); `U` shows its usages. Only direct usages are counted.
4. Usages are always shown as a pair: the **certain** ones and **up to** all of them (the upper bound of ADR 0037). Certain means resolved without doubt, or verified by level 2. The default order is by certain usages. References of kind `test` and references of a symbol to itself do not count.
5. A symbol without a usage gets an **assessment**, never a verdict: `approximate` if its language is in `ui.approximate_languages`, else `likely` if it is private, internal or protected, else `public`. The tab opens with a note that says what level 1 cannot see.
6. **Test code is left out by default** (one switch includes it). A symbol is test code if its file holds a `test` reference, if a folder, the file or an enclosing symbol is named like `conventions.test_folders` (`tests`, `benches`), or if such a name ends with one of them after an underscore (`search_tests`, `stats_tests.rs`).
7. The rankings hold types and callables. The complexity limit is `metrics.cyclomatic_medium_max`.
8. Two read-only commands, `stats.overview` and `stats.ranking` (`docs/server.md`). The pure ranking lives in the domain (`statistics`), the port `CodeStatistics` gives the symbols of a snapshot with their counts and measured values. The counts come from the reverse index that is in memory anyway, the rest from two reads. No new table, no migration, no new dependency.

## Limits (stated honestly)

- The rule for test code is a heuristic. A test function in a module with another name (for example `mod checks`) in a file without a `test` reference still appears among the unused symbols.
- The statistics are computed for every call. For CodeSkimr itself (about 17,800 symbols, 284,000 references) an answer takes about 0.3 s in a debug build. There is no budget for it yet; if large projects need one, the answer can become a read model of the snapshot.
- The HTML export does not contain the screen.
- A ranking shows at most 200 rows in the UI (the command allows 500); the hotspots are drawn from those rows.

## Consequences

- Blueprint 15.1 and 15.2 name the screen and its mock.
- Texts in `locales/*/stats.ftl`.
- The estimate "unused" becomes better with every recall fix of level 1 and exact with level 2; the screen needs no change for that.
