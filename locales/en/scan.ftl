# Scan and performance screen.

scan-title = Scan and performance
scan-state-running = Scan running
scan-state-done = Scan finished
scan-state-cancelled = Scan cancelled
scan-mode-full = full (first scan)
scan-mode-incremental = incremental
scan-progress = { $done } of { $total } files · about { $remaining } s left
scan-metric-files-per-second = Files/s
scan-metric-lines-per-second = Lines/s
scan-action-cancel = Cancel
scan-cancel-key = Esc
scan-metric-cores = Cores active
scan-metric-memory = Memory
scan-pipeline = Pipeline
scan-column-language = Language
scan-column-tier = Tier
scan-column-files = Files
scan-column-symbols = Symbols
scan-column-parse-time = Parse time
scan-budgets-title = Performance targets
scan-budgets-hint = Measured on every build
scan-recent-title = Recent scans
scan-hint-unchanged = Unchanged files are detected by hash and skipped.
scan-hint-responsive = Scan running · the interface stays usable
scan-hint-incremental-results = Results appear step by step

nav-scan = Scans and performance
scan-state-failed = Scan failed
scan-state-idle = No scan running
scan-idle-hint = Start a scan on the projects screen; its progress shows up here.
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
