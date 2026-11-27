# The screen "Conventions": the convention check of the console in the UI (ADR 0116).

nav-conventions = Conventions
conventions-title = Conventions
conventions-subtitle = Checks the files of the project folder against the code rules, as the code is now. No scan is needed and nothing is stored.
conventions-no-project = Pick a project first, the one whose code you want to check.
conventions-run = Check now
conventions-running = Checking …
conventions-run-hint = Reads every file of the project folder and checks it against the rules of length, complexity, parameters, literals and duplicates. A large project takes about as long as the analysis of a scan.
conventions-git-hint = This project comes from a Git address, so there is no folder on this machine to check.
conventions-folder = Folder: { $folder }
conventions-checked-at = Checked at { $time }
conventions-before-first = No result yet. "Check now" reads the files as they are now.
conventions-failed = The check could not be done.
conventions-copy = Copy as Markdown
conventions-copied = Copied
conventions-copy-failed = Could not copy
conventions-copy-none = There is nothing to copy before a check has run.
conventions-figure-files = Files checked
conventions-figure-findings = Findings
conventions-figure-unchecked = Not analysable
conventions-clean = No findings in { $count ->
    [one] { NUMBER($count) } file
   *[other] { NUMBER($count) } files
}.
conventions-col-rule = Rule
conventions-col-place = Place
conventions-col-symbol = Function
conventions-col-measure = Measured
conventions-col-measure-hint = The measure and what the rule allows
conventions-open = Open
conventions-open-hint = Opens the file at this line in the editor of the settings.
conventions-explorer = Explorer
conventions-explorer-gone = Not in the last scan, so the explorer cannot show it. Scan again.
conventions-explorer-hint = Shows the function in the explorer, as the last scan has it.
conventions-open-unavailable = The editor of the settings is not available, so the file cannot be opened from here.
conventions-measure-lines = { $value } lines, at most { $max }
conventions-measure-complexity = Complexity { $value }, at most { $max }
conventions-measure-parameters = { $value } parameters, at most { $max }
conventions-measure-literals = { $value } literals
conventions-measure-duplicate = { $percent } % like { $place }
conventions-not-checked-title = Not checked by this check, so a clean result says nothing about them:
conventions-unchecked-title = Files that could not be analysed:
conventions-markdown-head = Convention check of { $project }: { $findings } findings in { $files } files
conventions-markdown-none = No findings.
conventions-table-label = Findings of the check
conventions-more = { $hidden } more findings are not listed: the list holds { $shown } at most (mcp.max_limit).
