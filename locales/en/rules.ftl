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
rules-hold = Hold state
rules-hold-hint = "Hold state": existing violations count as old burdens. The CI fails only for new ones.
rules-hold-git = Folder projects only: a Git project has no project file to put the file next to.
rules-hold-done = { $count ->
    [one] State held on { $date }: 1 violation now counts as an old burden. Commit the file .codeskimr-baseline.toml.
   *[other] State held on { $date }: { $count } violations now count as old burdens. Commit the file .codeskimr-baseline.toml.
}
rules-hold-failed = The state could not be held.
rules-open = Open .codeskimr.toml
rules-open-hint = Opens the project file with the program your system names for it. If it is missing, it is created empty.
rules-open-git = Folder projects only: a Git project has no project file.
rules-open-created = The file { $file } did not exist and was created empty.
rules-open-failed = The project file could not be opened.
rules-layers-title = Layers
rules-layers-allowed = { $layer } → { $uses }
rules-layers-nothing = nothing
rules-cycles-title = No cycles between modules
rules-cycles-spec = cycles = "deny"
rules-ring-advice = Resolvable over { $edges ->
    [one] 1 edge
   *[other] { $edges } edges
} with { $places ->
    [one] 1 place
   *[other] { $places } places
}
rules-ring-show = Show in the graph
rules-add = Add rule
rules-add-git = Folder projects only: a Git project has no project file to write the rules into.
rules-add-forbid = Ban
rules-add-forbid-hint = Module A must not use module B
rules-add-public-api = Only through the public interface
rules-add-public-api-hint = The inside of a module stays closed
rules-add-cycles = Forbid cycles
rules-add-cycles-hint = A switch, no new card
rules-add-layers = Layers
rules-add-layers-hint = What each layer may use
rules-edit = Change rule
rules-delete = Delete rule
rules-dialog-forbid-new = Add ban
rules-dialog-forbid-edit = Change ban
rules-dialog-api-new = Add rule: only through the public interface
rules-dialog-api-edit = Change rule: only through the public interface
rules-dialog-layers = Layers: who may use whom
rules-dialog-delete = Delete rule?
rules-dialog-delete-body = The rule "{ $rule }" is removed from .codeskimr.toml. Violations of it are no longer reported.
rules-field-from = This module …
rules-field-to = … must not use this module
rules-field-reason = Reason (optional, shown on the page)
rules-field-module = Module
rules-field-internal = Inside (one prefix per line, each below the module)
rules-field-hint = From the namespaces of the scan ({ $count }); a prefix of your own can be typed.
rules-layers-help = A row uses the columns. Without "Limit" a layer may use anything; with it, only the ticked layers (none if nothing is ticked). Names and namespaces of the layers stay in the layer editor.
rules-layers-limit = Limit
rules-layers-uses = may use →
rules-layers-none = The project has no layers yet. Set them up in the layer editor first.
rules-preview-checking = Checking against the last scan …
rules-preview-none = With the last scan there would be no violation.
rules-preview-some = { $count ->
    [one] With the last scan this would be 1 violation ({ $old } already held).
   *[other] With the last scan this would be { $count } violations ({ $old } already held).
}
rules-preview-unscanned = The project has no scan yet, so there is no preview.
rules-preview-invalid = This is not a rule yet: check the names.
rules-promise = Writes into .codeskimr.toml. The other tables and comments stay; comments inside the rules tables are rewritten.
rules-cancel = Cancel
rules-save = Save
rules-saving = Saving …
rules-delete-confirm = Delete
rules-save-failed = The rules could not be saved.
rules-toml-label = Text written into the file
rules-tab-rules = Rules
rules-tab-layers = Layers
rules-tabs-label = Rules and layers
