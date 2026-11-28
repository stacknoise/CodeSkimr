# 0064 – Syntax colours in the source column

Status: accepted (Anton, 2026-11-20, decision card, option A: the server cuts the colours; the browser highlighter was considered and not chosen). Implemented in #484.

## Context

Every language pack ships a `highlights.scm` (blueprint 8.1, "syntax highlighting for the UI"), but nothing reads it: the source column of the explorer shows the code as plain text, and only the names of symbols are small elements (Alt+click, issues #158 and #171). Anton asked for coloured code on 2026-11-16. The blueprint names the file but not how its captures reach the screen, and the design tokens (15.3) have no colours for code.

## Options

1. **Colour in the browser** with a JavaScript highlighter. A new dependency, a second grammar for every language and a second source of truth next to the packs.
2. **Colour on the server** with the grammar and `highlights.scm` of the pack, and send the runs with the source. No new dependency (tree-sitter and the grammars are built in), one source of truth, the packs stay data.
3. **Colour at scan time** and store the runs. The index grows by every token of every file for something that is read for a few hundred lines at a time.

## Decision

Option 2.

1. `explorer.source` returns `tokens` next to `spans`: runs `{ line, start, end, class }` with the position in UTF-16 units like `spans`, in the order of the text, without overlaps. `class` is one of `comment`, `string`, `number`, `keyword`, `type`, `function`, `attribute`.
2. The port `SyntaxHighlighter` (application) and its adapter `TreeSitterHighlighter` (parsing) turn the captures of `highlights.scm` into classes: `@comment` is `comment`; `@string*` is `string`; `@number` and `@constant*` are `number`; `@keyword` and `@variable.builtin` are `keyword`; `@type*` and `@tag` are `type`; `@function*` is `function`; `@attribute` and `@label` are `attribute`. Any other capture, for example `@property` or `@variable.parameter`, stays plain. Where captures overlap, the later one wins.
3. The adapter reads only the grammar and `highlights.scm` of a language, when a file of that language is first shown. It never loads `symbols.scm`, `references.scm` or `metrics.scm`, so it adds nothing to the load of the packs that ADR 0063 budgets. A pack without `highlights.scm`, an unknown extension, a failed parse (200 ms limit) or a broken query give no tokens: the code is shown plain, never as an error. `highlights.scm` is not one of the files a pack can take from another pack (ADR 0056).
4. The lines are parsed as the excerpt of the declaration, not as the whole file. A fragment can colour a name wrongly where the grammar needs the context (a method without its class); keywords, strings, numbers and comments, which the colours mostly carry, do not depend on it.
5. Seven tokens per theme, `syn-keyword`, `syn-string`, `syn-number`, `syn-comment`, `syn-type`, `syn-function`, `syn-attribute`, in `design/tokens.toml`. They are text on `surface` and join the contrast rule of 15.3 (4.5:1). Petrol takes the values of Hell, as for the kind colours; Dunkel uses lighter variants. Comments are italic.
6. The names of symbols stay small elements with `data-symbol` and Alt+click; a name inside a coloured run keeps both.

## Consequences

- `docs/language-packs.md` describes the vocabulary of `highlights.scm`; `docs/ui.md` and `docs/server.md` the new field.
- The blueprint table of 15.3 and its contrast rule name the new tokens; a theme that misses one fails the `codegen` job.
- The packs that already ship a `highlights.scm` are coloured without a change; a pack that wants more classes later needs an ADR, because the classes are part of the wire format and of the tokens.
- The answer of `explorer.source` is larger by one run per token (the 400 line limit bounds it). The HTML export holds no source text and no tokens.

## Addendum: colours outside the source column

Code is shown in more places than the source column: signatures in the member table, the side panels of the explorer and of the usages, and the old and new signature in the view "Change". They get the same colours, from the same highlighter, through one read command, `code.highlight` (read right, no token). It takes `pieces` (`path`, `text`; the extension of `path` tells the language, nothing is read from the project folder) and returns, for each piece, the `tokens` as `explorer.source` does. At most 200 pieces of at most 600 characters; anything beyond, an invalid path or a language without `highlights.scm` comes back plain. The web UI has one component for it (`shared/code/Code.tsx`) that batches the pieces asked for in the same moment, caches the answers and shows the code plain until they arrive or when the call fails (the HTML export has no such command, so it stays plain). A signature is parsed as a fragment (point 4), so keywords and types are coloured, a name may stay plain.
