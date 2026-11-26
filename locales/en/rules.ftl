# The screen "Rules": architecture rules and their violations (ADR 0104).

nav-rules = Rules
rules-title = Rules
rules-subtitle = from .codeskimr.toml · checked against the last scan
rules-no-project = Pick a project first, the one whose rules you want to see.
rules-not-scanned = The project has not been scanned yet, so no rule can be judged. Scan it first.
rules-load-failed = The rules of the project could not be read.
rules-none = The project file has no rules yet. Write them in the table [rules] of the .codeskimr.toml of the project; see the manual.
rules-kept-count = { $count } kept
rules-kept-label = Rules
rules-failing-count = { $count } fail the CI
rules-failing-label = new violations
rules-held-count = { $count } held on { $date }
rules-held-label = old burdens
rules-held-none = No state held
rules-held-none-label = every violation counts
rules-resolved-count = { $count } resolved
rules-resolved-label = since the state was held
rules-kind-new = new rule kind
rules-forbid-title = Ban: { $from } must not use { $to }
rules-forbid-spec = forbid = { "{" } from = "{ $from }", to = "{ $to }" { "}" }
rules-public-api-title = Only through the public interface
rules-public-api-spec = { $internal } only within { $module }
rules-state-kept = kept
rules-badge-new = { $count } new
rules-badge-old = { $count ->
    [one] 1 old burden
   *[other] { $count } old burdens
}
rules-violation-old = old burden
rules-violation-new = new
rules-places = { $count ->
    [one] 1 place
   *[other] { $count } places
}
rules-more = … { $count } more
rules-show-all = Show all
rules-show-less = Show fewer
rules-cycles-pending = Cycles between modules are switched on in the project file. The check follows in a later version.
rules-hold = Hold state
rules-hold-hint = "Hold state": existing violations count as old burdens. The CI fails only for new ones.
rules-hold-git = Folder projects only: a Git project has no project file to put the file next to.
rules-hold-done = { $count ->
    [one] State held on { $date }: 1 violation now counts as an old burden. Commit the file .codeskimr-baseline.toml.
   *[other] State held on { $date }: { $count } violations now count as old burdens. Commit the file .codeskimr-baseline.toml.
}
rules-hold-failed = The state could not be held.
rules-layers-title = Layers
rules-layers-allowed = { $layer } → { $uses }
rules-layers-nothing = nothing
