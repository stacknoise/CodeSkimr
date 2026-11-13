# The HTML export (blueprint 10.2): the one page that is read without a server.

export-banner = Export of { $project } · scan { $snapshot } · { $date }
export-banner-commit = commit { $commit }
export-usages-state = Usages: { $verified } verified, { $unchecked } unchecked. Unchecked usages were found by name resolution only and may be incomplete or wrong.
export-limits = Static export: the source code is not included, and the scan cannot be repeated here.
export-nav-label = Export
