# Rules

The screen **Rules** (left bar) checks the architecture rules of your project file `.codeskimr.toml` against the last scan and lists their violations. The layers have a screen of their own, **Layers**, in the left bar (the editor of the layers, see [Projects and scan](projects-and-scan.md)).

## What a rule is

| Rule | Meaning | In the file |
|---|---|---|
| Layers | a layer may use only the layers in its `may_use` | `[[layers]]` with `may_use` |
| No cycles | modules must not depend on each other in a ring | `[rules]` `cycles = "deny"` |
| Ban | module A must not use module B | `[[rules.forbid]]` with `from`, `to`, optional `reason` |
| Public interface | the inside of a module may be used from inside only | `[[rules.public_api]]` with `module`, `internal` |

A violation is a symbol that reaches a symbol it must not reach, with every place. Both names are buttons that show the symbol in the explorer; a ring has the button **Show in the explorer**. A name that the last scan does not know is off, with the hint to scan again. **Hold state** records all current violations as old burdens; the CI (`codeskimr rules`) then fails only for new ones.

## Creating and changing rules in the UI

You do not have to write the file by hand (ADR 0112). **Add rule** opens a menu:

- **Ban** and **Only through the public interface** open a dialog. The module fields offer the namespaces of the last scan; you can also type a prefix of your own. Under the form the dialog shows how many violations the last scan would find with the rule as typed (and how many of them are already held), and the text that is written into the file. **Save** writes the rule at once.
- **Forbid cycles** is a switch; it is off in the menu once it is on.
- **Layers** opens a table "who may use whom": a row uses the columns. Without the tick "Limit" a layer may use anything; with it, only the ticked layers. Names and namespaces of the layers stay in the tab **Layers**.

Every rule card has a pen (change) and a bin (delete; a dialog asks first). Only the `rules` tables (and the `may_use` of the layers) are rewritten; everything else in the file stays, and so do comments outside the rules tables. An invalid rule (a module inside the other, the same pair twice, an unknown layer) is refused with `CFG-101` and the file stays as it was.

A Git project has no project folder, so **Add rule**, the pen and the bin are off there; add the rules to the file in the repository.

## Checking the code rules on the screen "Conventions"

The screen "Conventions" (its own item in the left bar, next to "Rules") checks the code of the project folder against the code rules, as the console command `codeskimr check` does (ADR 0116): a function that is too long, too complex or has too many parameters, literals in the code, and two functions that are copies of each other (`docs/conventions.md`). It reads the files as they are now, so it also sees edits that no scan has seen. Nothing is stored.

**Check now** starts it for the whole folder of the project. A large project takes about as long as the analysis of a scan. The strip shows how many files were checked, how many findings there are and how many files could not be analysed. The findings are in a table that you can sort by every column and whose columns you can widen; **Open** opens the file at the line in the editor of the settings. **Copy as Markdown** puts the result into the clipboard, for an issue or a comment. Below the table the tab names the rules this check does not cover (a clean result says nothing about them) and the files it could not read.

A project from a Git address has no folder on this machine, so the button is off there.
