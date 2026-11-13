# Explorer and search

The explorer shows the active project ([Projects and scan](projects-and-scan.md)) from its newest snapshot.

## Tree

Layer, namespace, type; below a type the groups "Fields etc." and "Methods" and nested types with their members. Types that belong to no layer come last. At the start everything is closed and the overview of the project is shown.

- A click on the caret only opens or closes. A click on a type that is not selected selects it; a click on the selected type opens or closes it. Groups open and close on their own. Selecting never changes what else is open.
- The filter above the tree keeps the types whose name contains the text, with their layer and namespace opened.
- Keys: `↑` `↓` move, `→` opens (or goes to the first child), `←` closes (or goes to the parent), `Home` and `End` jump, `Enter` and `Space` select or toggle, `Ctrl+←` closes everything, `Ctrl+.` shows the selection.

The tree is virtual: it stays fast with hundreds of thousands of entries.

## Details

For the selected type: breadcrumb, name with kind and visibility, documentation, ten metric tiles and a table of members (chips All, Properties, Methods, "public only").

The metrics are lines of code, cyclomatic complexity, maintainability index, instability and more. Each one is coloured good, medium or poor against limits from the configuration (`metrics.*` in `config/default.toml`). **The number is always shown next to the colour**; the tooltip of a column names the limits.

Basic-tier languages ([overview](index.md)) have no maintainability index and no documentation or visibility.

## Source column

The right column shows file, position and code of the selected member (or of the type). The code is read from the project folder **as it is now**, not from the scan: if you edited the file since, lines can be shifted until the next scan. Only a project that is a folder on your machine has source text; for a Git project the column says so. A file that is gone gives `SRC-201`, a file over 4 MiB `SRC-501`; a declaration longer than 400 lines is cut with a note. Paths outside the project folder are refused, also through links.

Reading source text needs the installation token. The browser has it after `codeskimr open`; nothing for you to do.

## Quick search

`Ctrl+K` (`Cmd+K` on a Mac) or the search field in the top bar opens the search over the active project.

- Type to find types and members by name (fuzzy). A prefix narrows it: `t:` types, `m:` members, `>` commands (go to a screen, scan the project again). The chips All, Types, Members and Commands do the same.
- `↑` `↓` choose, `Enter` opens the hit in the explorer, `Tab` and `Shift+Tab` change the chip, `Ctrl+C` copies the qualified name when no text of the input is selected, `Esc` closes.
- At most 50 hits are shown, best first. A project without a scan gives no hits.

## Footer

The footer of every screen shows the connection, the number of projects and types, how long the program took to start, its memory, and the language switch `EN | DE`. It only shows measured values.

## Open in editor

The explorer can open a member in your editor through a link. The template is `editor.url_template` (and the allowed link schemes `editor.allowed_schemes`) in the settings or the configuration; the settings screen has a field for it. These buttons are off until the position of a hit is available.
