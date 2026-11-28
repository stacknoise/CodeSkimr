# Scan and performance screen.

scan-title = Scan and performance
scan-state-running = Scan running
scan-state-done = Scan finished
scan-state-cancelled = Scan cancelled
scan-mode-full = full
scan-mode-first = full (first scan)
scan-mode-incremental = incremental
scan-progress = { $done } of { $total } files · about { $remaining } s left
scan-metric-files-per-second = Files/s
scan-metric-lines-per-second = Lines/s
scan-action-cancel = Cancel
scan-cancel-key = Esc
scan-metric-cores = Cores active
scan-metric-memory = Memory
scan-progress-label = Progress of the scan
scan-pipeline = Pipeline
scan-column-language = Language
scan-column-tier = Tier
scan-column-files = Files
scan-column-symbols = Symbols
scan-column-parse-time = Parse time
scan-budgets-title = Performance targets
scan-budgets-hint = Measured on every build; “—” means not measured yet in this session
scan-recent-title = Recent scans
scan-hint-unchanged = Unchanged files are detected by hash and skipped.
scan-hint-responsive = Scan running · the interface stays usable
scan-hint-incremental-results = Results appear step by step

nav-scan = Scans and performance
scan-state-failed = Scan failed
scan-state-idle = No scan running
scan-idle-hint = Start a scan on the projects screen; its progress shows up here.
scan-state-starting = Scan starting
scan-starting-hint = The scan is running. The progress shows up with the next report of the server.
scan-stage-unmeasured = Running; the progress of this step cannot be measured
scan-no-project = Add a project first, then its scans are listed here.
scan-files-done = { $done } of { $total } files
scan-pending = waiting
scan-column-lines = Lines
scan-unavailable = —
scan-stage-fetch = Fetch source
scan-stage-fetch-detail = Make the source available locally
scan-stage-discover = Find files
scan-stage-discover-detail = Respect .gitignore, detect languages, hash contents
scan-stage-analyze = Parse
scan-stage-analyze-detail = Tree-sitter, in parallel
scan-stage-resolve = Resolve dependencies
scan-stage-resolve-detail = Imports, name resolution, layers
scan-stage-measure = Metrics
scan-stage-measure-detail = Lines, complexity, maintainability
scan-stage-persist = Save
scan-stage-persist-detail = SQLite, search index
scan-step-line = { $step } · { $number } / { $of } · { $time } so far
scan-steps-title = Time per step of saving
scan-steps-total = together
scan-steps-unit = { $seconds } s
scan-step-header = Create the scan
scan-step-prune-before-load = Remove old scans first
scan-step-facts = Save files and symbols
scan-step-references = Save references
scan-step-reference-indexes = Build reference indexes
scan-step-fingerprints = Save similarity features
scan-step-details = Save rules, warnings and parse times
scan-step-integrity = Check the result
scan-step-read-models = Compute overviews and totals
scan-step-prune = Delete surplus old scans
scan-step-commit = Finish
scan-budget-cold-start = Program start
scan-budget-search-keystroke = Search (keystroke)
scan-budget-explorer-switch = Switch type
scan-budget-usages-query = Usages query
scan-budget-graph-layout = Graph layout
scan-budget-full-scan = Scan 100k lines
scan-budget-incremental = Incremental
scan-budget-target = Target < { $ms } ms
scan-budget-measured = { $ms } ms
scan-recent-line = { $name } · { $mode }
scan-recent-result = { $files ->
    [one] { NUMBER($files) } file
   *[other] { NUMBER($files) } files
} · { $when }
scan-memory-value = { $mb } MB
scan-stage-counts = { $text } · { $done } / { $total }
scan-stage-time = { $seconds } s
scan-tier-full = full
scan-tier-basic = basic
scan-left-out-title = Files left out
scan-left-out-hint = These files could not be analysed. The rest of the project was scanned.
scan-left-out-missing = missing in the result
scan-left-out-kept = shown as the previous scan saw it
scan-left-out-more = { $count ->
    [one] and one more file
   *[other] and { $count } more files
}
scan-left-out-unnamed = { $count ->
    [one] One file was left out. Its name shows after the next scan.
   *[other] { $count } files were left out. Their names show after the next scan.
}
scan-files-unchanged = No file changed · { $checked } checked

scan-history-keep = { $count ->
    [one] { NUMBER($count) } state is kept
   *[other] { NUMBER($count) } states are kept
}
scan-history-column-time = Time
scan-history-column-mode = Kind
scan-history-column-duration = Duration
scan-history-column-files = Files
scan-history-column-changed = Changed
scan-history-current = current
scan-history-empty = No scan yet.
scan-history-note = The duration is stored per scan. Every scan is an entry of its own; a scan without changes shows 0 changed files and confirms the state before it.
scan-history-last = Last scan
scan-history-files-changed = { $count ->
    [one] { NUMBER($count) } file changed since the one before
   *[other] { NUMBER($count) } files changed since the one before
}
scan-history-average = Average
scan-history-average-hint = last { $count } scans
scan-history-compare = Compare with current
scan-history-compare-title = Compare this scan with the current one in the view Change
scan-history-failed = The last scans could not be read.
