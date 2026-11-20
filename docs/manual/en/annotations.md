# Notes, tasks and critical marks

You can put three kinds of **annotations** on a symbol (a type or a member). They are stored on your computer, next to the project, and they survive a scan: if the symbol is still there, its annotations stay with it.

| Kind | Use it to |
|---|---|
| **Critical mark** | say "this symbol is system critical"; a change to it raises an alert after the next scan |
| **Note** | write a remark in plain text |
| **Task** | track work on a symbol: title, optional text, status (open, in progress, done), due date and assignee |

Notes are stored as plain text. Do not write passwords or keys into them.

## In the explorer

Select a type or a member. The side panel has the tabs **Info**, **Critical**, **Notes** and **Tasks** (arrow keys switch between them). Three keys act on the selection and are also entries of the quick search (`Ctrl+K`):

- `K` marks the selection as critical or takes the mark off (with an optional reason).
- `N` adds a note.
- `T` adds a task.

Critical symbols have a stripe and the label "Critical" in the member table, a shield in the tree (a folded group shows how many critical symbols it holds), and a shield and label in the details. Critical is never shown by colour alone. The switch "Show annotations" puts the texts of notes and tasks under the rows. The [dependency map](dependency-map.md) has the switches "Highlight critical" and "only critical and their neighbours".

## Alerts

After a scan, CodeSkimr compares every critical symbol with its last confirmed state. The bell in the top bar shows how many alerts are open. Three kinds exist:

- **Changed:** the content, the signature, the complexity or the lines of code changed.
- **Removed:** the symbol no longer exists; the alert says how many places still refer to it.
- **Orphaned:** the target was not found; the alert names the closest symbol. "Reassign" moves the annotation to it, "Remove mark" drops the mark.

The drawer of the bell has the tabs Open, Changed, Removed (orphaned targets are listed here) and Dismissed. The cross dismisses one alert, "Dismiss all open alerts" dismisses all of a tab after one question. A click jumps to the symbol in the explorer. `Esc` closes the drawer.

## The board

The screen **Annotations** shows all tasks of the project in one column per status; a task changes its column with the status list of its card. The filters are "Mine", "Only critical" and "Due". "Mine" works after you have added an annotation in this browser, because that is how CodeSkimr learns your name. A second view lists the notes.

## On the command line

All commands take the project as its id or as the folder or Git address it was scanned from. `--json` prints the answer as JSON; without it you get one line per row. Lists are paged: `--limit` and `--offset`, and the text says how to continue.

```
codeskimr annotations list <project> [--kind critical|note|task] [--status open|in_progress|done] [--symbol <name> --symbol-kind <kind> [--param <type>]...] [--limit N] [--offset N] [--json]
codeskimr annotations add <project> --kind critical|note|task --symbol <name> --symbol-kind <kind> [--param <type>]... [--title <title>] [--text <text>] [--status <status>] [--due <seconds>] [--assignee <name>] [--json]
codeskimr annotations update <project> <id> [--symbol <name> --symbol-kind <kind>] [--title ...] [--text ...] [--status ...] [--due ...] [--assignee ...] [--clear-text] [--clear-due] [--clear-assignee] [--json]
codeskimr annotations remove <project> <id> [--json]
codeskimr alerts list <project> [--state open|dismissed] [--kind changed|removed|orphaned]... [--limit N] [--offset N] [--json]
codeskimr alerts dismiss <project> <id> [--json]
codeskimr alerts dismiss-all <project> [--kind changed|removed|orphaned]... --yes [--json]
```

- `--symbol` is the qualified name, `--symbol-kind` its kind (`class`, `method`, `namespace` and so on). For overloads repeat `--param` once per parameter type.
- `--title` belongs to tasks. `--due` is a point in time in seconds since the Unix epoch.
- `alerts list` shows the open alerts unless you ask for `--state dismissed`. `alerts dismiss-all` does nothing without `--yes`.
- Exit code `0`, or `1` with the error code on standard error ([Error codes](errors.md)).

Example:

```
codeskimr annotations add . --kind task --symbol Shop.Orders.OrderService.Place --symbol-kind method --title "Check rounding" --assignee anna
codeskimr alerts list . --kind changed
```

## For code agents

The MCP tools `list_annotations` and `get_alerts` read the same data ([MCP setup](mcp.md)). Agents can read annotations and alerts but not change or dismiss them.

## When a project is removed

Removing a project deletes its annotations and alerts too ([Projects and scan](projects-and-scan.md)). The [HTML export](export.md) does not contain them.
