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
rules-cycles-title = No cycles between modules
rules-cycles-spec = cycles = "deny"
rules-ring-modules = { $count } modules
rules-ring-advice = Weakest edge { $from } → { $to } with { $weakest ->
    [one] 1 reference
   *[other] { $weakest } references
}. Resolvable over { $edges ->
    [one] 1 edge
   *[other] { $edges } edges
} with { $places ->
    [one] 1 place
   *[other] { $places } places
}.
