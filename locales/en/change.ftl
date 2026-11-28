# The screen "Change": what the difference between two scans hits (ADR 0097, mocks 29 and erw-auswirkung).

nav-change = Change
change-view-title = Change
change-view-subtitle = What does this change hit?
change-view-no-project = Pick a project first, the one whose scans you want to compare.
change-view-load-failed = The comparison could not be read.
change-view-reading = Comparing the scans …
change-view-too-few = The project has fewer than two scans, so there is nothing to compare yet.
change-view-tabs-label = Views of the change
change-view-tab-impact = Impact
change-view-tab-compare = Comparison
change-view-panel-label = The comparison of two scans
change-view-compared = Comparison

change-view-base = Base (before)
change-view-target = Target (after)
change-view-run = Compare
change-view-scan-option = Scan of { $time }
change-view-scan-current = Current scan

change-view-copy = Copy as Markdown
change-view-copy-done = Copied
change-view-copy-failed = Not copied

change-view-tile-symbols = changed symbols
change-view-tile-public = of them public
change-view-tile-usages = affected usages
change-view-tile-usages-certain = { $count } sure
change-view-tile-open = places not adapted
change-view-tile-files = affected files

change-view-col-symbol = Symbol
change-view-col-change = Change
change-view-col-visibility = Visible
change-view-col-usages = Uses
change-view-col-state = State
change-view-col-file = File

change-view-kind-removed = removed
change-view-kind-added = new
change-view-kind-moved = moved
change-view-kind-signature = Signature
change-view-kind-visibility = Visibility
change-view-kind-content = Content

change-view-visibility-public = public
change-view-visibility-protected = protected
change-view-visibility-internal = internal
change-view-visibility-private = private

change-view-state-open = { $count } open
change-view-state-adapted = adapted
change-view-state-none = no checklist

change-view-table-label = Symbols the change touches
change-view-none = The two scans have no difference in symbols.
change-view-same = Nothing changed: both scans have the same state, because no file changed between them.

change-view-detail-signature = Changed signature
change-view-detail-visibility = Changed visibility
change-view-detail-removed = Removed symbol
change-view-detail-added = New symbol
change-view-detail-content = Changed content
change-view-detail-moved = Moved symbol
change-view-detail-narrowed = The visibility got narrower.
change-view-detail-no-checklist = Only the content changed, or the symbol is new: its usages cannot break by this change, so no checklist is made.
change-view-detail-label = Usages of the selected symbol
change-view-usages-count = { $count ->
    [one] { NUMBER($count) } usage
   *[other] { NUMBER($count) } usages
}
change-view-usages-certain = { $count } sure
change-view-usages-probable = { $count } probable
change-view-usages-cut = { $shown } of { $total } usages are listed.
change-view-usages-all-adapted = No place is still open: every usage was adapted in the change.
change-view-filter-open = Only open · { $count }
change-view-filter-all = All · { $count }
change-view-filter-label = Which usages are listed
change-view-group-open = { $count } open
change-view-group-adapted = adapted
change-view-status-not-touched = not touched
change-view-status-adapted = adapted in the change
change-view-confidence-certain = sure
change-view-confidence-probable = probable
change-view-checklist-note = "Not touched" is the checklist: places that use the changed symbol but were not changed in this difference.

change-view-violations = { $count ->
    [one] { NUMBER($count) } new layer violation
   *[other] { NUMBER($count) } new layer violations
}
change-view-cycles = { $count ->
    [one] { NUMBER($count) } new layer cycle
   *[other] { NUMBER($count) } new layer cycles
}
change-view-edge = { $from } uses { $to } ({ $count } references)
change-view-critical = System critical
change-view-critical-line = { $name } is marked as system critical and was { $outcome }.
change-view-outcome-removed = removed
change-view-outcome-moved = moved
change-view-outcome-changed = changed
change-view-cut = The answer was cut to the configured limit; the numbers count the listed entries.

change-view-card-added = New
change-view-card-added-hint = symbols
change-view-card-changed = Changed
change-view-card-changed-hint = Signature { $signature } · Visibility { $visibility } · Content { $content }
change-view-card-changed-hint-cut = Of the { $shown } listed: Signature { $signature } · Visibility { $visibility } · Content { $content }
change-view-card-removed = Removed
change-view-card-removed-hint = check the callers
change-view-card-moved = Moved
change-view-card-moved-hint = to another file
change-view-list-label = The symbols of the difference
change-view-compare-note = Two states of the same project. "Impact" shows the places that are hit.

change-view-filter-bar-label = Filter the symbols
change-view-filter-query-label = Search by qualified name
change-view-filter-query-placeholder = Search by name
change-view-filter-cause-label = Kind of change
change-view-filter-cause-all = All changes
change-view-filter-cause-added = New
change-view-filter-cause-changed = Changed
change-view-filter-cause-removed = Removed
change-view-filter-cause-moved = Moved
change-view-filter-kind-label = Kind of symbol
change-view-filter-kind-all = All kinds
change-view-filter-public = Only public
change-view-filter-only-open = Only with open places
change-view-filter-reset = Reset filter
change-view-filter-reset-off = No filter is set, so there is nothing to reset.
change-view-filter-only-open-off = The comparison has no checklist, so it has no open places.
change-view-filter-count = { $shown } of { $listed }
change-view-filter-count-cut = { $shown } of { $listed } listed ({ $total } in all)
change-view-filter-empty = No symbol matches the filter.

# The view "Metrics" (ADR 0114, mock 43 a).
change-view-tab-metrics = Metrics
change-metrics-reading = Reading the metrics …
change-metrics-load-failed = The metrics could not be read.
change-metrics-card-worse = Got worse
change-metrics-card-worse-hint = objects, { $crossing } past a limit
change-metrics-card-better = Got better
change-metrics-card-better-hint = objects, { $crossing } back under a limit
change-metrics-card-same = Unchanged
change-metrics-card-same-hint = not listed
change-metrics-card-same-unknown = a list was cut
change-metrics-card-new = Not comparable
change-metrics-card-new-hint = new { $added } · removed { $removed }
change-metrics-filter-label = Metric
change-metrics-filter-all = All
change-metrics-metric-loc = Lines
change-metrics-metric-cc = Complexity
change-metrics-metric-mi = Maintainability
change-metrics-metric-ce = Coupling out
change-metrics-metric-instability = Stability
change-metrics-level-label = Level
change-metrics-level-member = Members
change-metrics-level-type = Types
change-metrics-level-module = Modules
change-metrics-list-worse = Got worse
change-metrics-list-better = Got better
change-metrics-col-object = Object
change-metrics-col-values = From → to
change-metrics-unavailable = Members have no such metric; it exists for types and modules.
change-metrics-card-new-unknown = comparison not read
change-metrics-copy-waiting = Available once the metrics are read.
change-metrics-copy-failed = The metrics could not be read, so there is nothing to copy.
change-metrics-list-label = Objects whose metrics moved
change-metrics-crossing-into = past the limit
change-metrics-crossing-out = back under the limit
change-metrics-none = No object moved by its threshold.
change-metrics-note = Compared are types, members and modules that exist in both scans (the same key as in the comparison). New and removed objects are in the view "Comparison". A move counts only from a threshold (settings compare.metric_*), so that a change of one line is not a decline.
