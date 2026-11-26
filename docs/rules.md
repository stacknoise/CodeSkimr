# Architecture rules (ADR 0104 to 0107)

The project file `.codeskimr.toml` may hold the table `rules` besides the layers. The rules read the certain references of the newest scan (not those of kind `test`, not those that match several symbols), so they see the state of the last scan.

```toml
[rules]
cycles = "deny"            # "off" (default) or "deny", see ADR 0105; built in a later step

[[rules.forbid]]           # ADR 0106: Shop.Reports must not use Shop.Checkout
from = "Shop.Reports"
to = "Shop.Checkout"
reason = "reports read the read model only"   # optional

[[rules.public_api]]       # ADR 0107: the inside of Shop.Data is for Shop.Data only
module = "Shop.Data"
internal = ["Shop.Data.Internal"]
```

A prefix equals a name or lies above it up to a separator (`.` or `::`), the same as for layers. A `forbid` rule whose two prefixes contain each other, and a `public_api` rule whose `internal` is empty or not below `module`, are refused with `CFG-101` on the key `rules`.

A violation is the symbol that refers and the symbol it must not reach, with every place of the reference. Its identity is the rule and the two names, never the line.

## In the terminal and for agents

`codeskimr rules [--json]` and the MCP tool `check_rules` (`docs/cli.md`, `docs/mcp.md`) give the answer of the screen. The command exits with 1 only if there is a violation the held state does not have, so a CI job runs `codeskimr scan .` and then `codeskimr rules`. A violation has the code `CHK-107`.

## Holding the state

`codeskimr_application::rules::Baseline` is the content of `.codeskimr-baseline.toml` (`held_on` and one entry with `rule`, `from` and `to` per held violation). A violation with an entry is an old burden; one without is new. Entries whose violation is gone are reported as resolved. The file is meant to be committed.

**Holding the state** replaces the whole file with every violation of the newest scan, with today's date (UTC, `YYYY-MM-DD`) in `held_on`: the button "Hold state" of the screen Rules, the command `rules.hold` (right to manage, like `unused.keep`) and `codeskimr rules --hold` do the same, so the console and the UI can do the same. Only a folder project has a place for the file; a Git project is refused with `PRJ-101`, a project without a scan with `MCP-203`. Entries whose violation is gone disappear this way, so the number of old burdens only goes down.

## The layers

If at least one layer of the project has a `may_use`, the layer rule (`CHK-103`, ADR 0054) is the first rule of the screen, the command and the tool, with the id `layers`. A violation is one referring symbol and one symbol of a layer it may not use, with every place, so it can be held like the others. `codeskimr check` and `check_conventions` keep reporting `CHK-103` for the files they look at.
