# The HTML export (blueprint 10.2): the one page that is read without a server.

export-banner = Export of { $project } · scan { $snapshot } · { $date }
export-banner-commit = commit { $commit }
export-usages-state = Usages: { $verified } verified, { $unchecked } unchecked. Unchecked usages were found by name resolution only and may be incomplete or wrong.
export-limits = Static export: the source code is not included, and the scan cannot be repeated here.
export-nav-label = Export

# The button in the top bar and its quick-search entry.
export-button-label = Export HTML
export-button-title = Download the newest scan as one self-contained HTML page. The source code is not included.
export-button-needs-scan = Scan the project first, then it can be exported.
export-button-busy = Exporting …
export-button-failed = The export failed.
export-command-html = Export project as HTML
