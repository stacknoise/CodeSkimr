# Texts of the comparison of two scans and of the impact of the difference (ADR 0097).

compare-direct-only-note = Only direct usages are known; reflection, strings and other repositories are not seen by the comparison.

compare-scans-head = Scans of the project: { $count } kept (up to { $keep }), average duration { $average }.
compare-scans-none = The project has no scan yet.
compare-scans-line = { $snapshot }  { $created }  { $mode }  { $files } files, { $changed } changed, { $duration }  { $commit }
compare-scans-steps = Saving took: { $steps }
compare-scans-step = { $step } { $ms } ms
compare-duration = { $ms } ms
compare-unknown = -

compare-head = Comparison of scan { $base } ({ $base_time }) with scan { $target } ({ $target_time }).
compare-counts = { $added } added, { $removed } removed, { $changed } changed, { $moved } moved.
compare-added = Added ({ $shown } of { $total }):
compare-removed = Removed ({ $shown } of { $total }):
compare-changed = Changed ({ $shown } of { $total }):
compare-moved = Moved ({ $shown } of { $total }):
compare-symbol-line = { $kind } { $name }  { $path }
compare-changed-line = { $kind } { $name }  { $path }  ({ $reasons })
compare-moved-line = { $kind } { $name }  { $from } -> { $path }
compare-narrowed = narrowed
compare-critical = Critical symbols the difference touches:
compare-critical-line = { $kind } { $name }  { $outcome }
compare-violations = New layer violations:
compare-cycles = New layer cycles:
compare-edge-line = { $from } -> { $to }  ({ $count } references)
compare-cut = The lists were cut to the configured limit; use --json for the totals.

change-head = Change from scan { $base } to scan { $target }: { $changed } symbols changed or removed, { $public } of them public.
change-usages = Usages: { $certain } sure, { $probable } probable, { $open } not touched.
change-symbol = { $kind } { $name }  { $path }  ({ $cause })
change-usage-line = { $line }  ({ $status })
change-cause-removed = removed
change-cause-changed = changed: { $reasons }
change-status-adapted = adapted
change-status-not-touched = not touched
change-no-impact = The difference changes no signature or visibility and removes no symbol: no usage can break.

change-md-title = ## CodeSkimr: change from scan { $base } to scan { $target }
change-md-changed = - Changed symbols: { $changed }, of them public: { $public }
change-md-usages = - Usages: { $certain } sure, { $probable } probable
change-md-open = - Not touched: { $open }
change-md-checklist = ### Not touched: check these places
change-md-item = - [ ] `{ $path }:{ $line }` { $kind } of `{ $symbol }` ({ $cause }), in `{ $holder }`
change-md-none = Every usage that the change can break is adapted in the same change.
change-md-violations = ### New layer violations
change-md-cycles = ### New layer cycles
change-md-edge = - `{ $from }` -> `{ $to }` ({ $count } references)
change-md-critical = ### Critical symbols the change touches
change-md-critical-line = - { $kind } `{ $name }`: { $outcome }
change-md-cut = The answer was cut to the configured limit; the numbers count the listed entries.
change-md-note = > { $note }

metrics-head = Metric changes from scan { $base } ({ $base_time }) to scan { $target } ({ $target_time }), level { $level }: { $worse } worse, { $better } better, { $looked } objects looked at.
metrics-worse = Worse ({ $shown } of { $total }):
metrics-better = Better ({ $shown } of { $total }):
metrics-line = { $kind } { $name }  { $path }  { $moves }
metrics-move = { $metric } { $from } -> { $to }
metrics-move-crossing = { $move } ({ $crossing })
metrics-name-loc = lines
metrics-name-cc = complexity
metrics-name-mi = maintainability
metrics-name-ce = coupling out
metrics-name-instability = instability
metrics-into-poor = now rated poor
metrics-out-of-poor = no longer rated poor
metrics-none = No object moved by its threshold.
