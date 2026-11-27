# Command line interface.

cli-error-line = error[{ $code }]: { $message }
cli-scan-started = Scanning { $path } …
cli-scan-finished = Scanned { $files } files in { $seconds } s.
cli-scan-tool-missing = Tool missing for { $languages }: { $tool } – { $purpose } { $install }
cli-scan-tool-not-runnable = Tool found but it does not start, needed for { $languages }: { $tool } – { $found } { $purpose } { $install }
cli-scan-tool-not-runnable-silent = Tool found but it does not start, needed for { $languages }: { $tool } (it printed no error) – { $purpose } { $install }
cli-scan-tool-too-old = Tool too old for { $languages }: { $tool } { $found } is installed, { $required } is required – { $purpose } { $install }
cli-scan-tool-too-new = Tool too new for { $languages }: { $tool } { $found } is installed, which is more than the project allows (it names { $required }) – { $purpose } { $install }
cli-scan-tool-unknown = Tool version not readable for { $languages }: { $tool } (required: { $required }) – { $purpose } { $install }
cli-unknown-command = Unknown command: { $command }
cli-help-hint = Run with --help to see all commands.
cli-codes-not-found = No error code with the id { $id } exists.
cli-scan-details = { $symbols } symbols, { $references } references, snapshot { $snapshot }.
cli-scan-stage = { $stage } { $seconds } s
cli-scan-stages = Steps: { $stages }.
cli-about = CodeSkimr turns code repositories into navigable documentation.
cli-cmd-scan = Scan a folder or a Git URL and print a summary
cli-cmd-mcp = Serve the MCP protocol over standard input and output
cli-cmd-serve = Start the local server without opening a browser
cli-serve-listening = Listening on { $url }. Stop with Ctrl+C.
cli-start-keep-open = Keep this window open as long as you use CodeSkimr.
cli-start-tray = CodeSkimr runs in the system tray now. The browser opens in a moment; the first start can take longer. Open the menu of the tray icon to show the browser or the log, or to quit. Log: { $log }
cli-arg-console = Stay in this console window instead of running in the system tray
cli-tray-open = Open CodeSkimr
cli-tray-log = Show log
cli-tray-quit = Quit CodeSkimr
cli-tray-tooltip = CodeSkimr
cli-arg-source = Folder or Git URL to scan
cli-arg-branch = Branch of a Git source (default: main)
cli-arg-json = Print the summary as JSON
cli-arg-project = Folder of the scanned project (default: the current folder)
cli-arg-data-dir = Folder for the database and the Git cache
cli-arg-config = User configuration file
cli-arg-set = Override a setting for this run, for example metrics.cyclomatic_good_max=4
cli-arg-languages = Folder with the language packs
cli-credentials-hint = Secrets in a Git URL (user info, query or fragment) are not accepted: they would be stored and printed. Use a Git credential helper or an SSH key.
cli-arg-watch = Keep running after the scan and scan again when files change
cli-watch-started = Watching for changes. Press Ctrl+C to stop.
cli-watch-updated = { $changed } changed files scanned again in { $millis } ms.
cli-cmd-check = Check files against the code rules (length, duplicates, cognitive complexity, parameters, literals) and exit with 1 if there are findings
cli-arg-paths = Files or folders to check, relative to the current folder (default: everything)
cli-check-finding = { $path }:{ $line }: { $code } { $message }
cli-check-too-complex = { $name } has the cognitive complexity { $value }, at most { $limit } is allowed.
cli-check-too-many-parameters = { $name } has { $value } parameters, at most { $limit } are allowed.
cli-check-literals = { $name } contains { $count ->
    [one] one literal
   *[other] { $count } literals
}; values belong in constants, configuration or message files.
cli-check-layer = { $name } in layer { $from_layer } uses layer { $to_layer } ({ $count ->
    [one] one reference
   *[other] { $count } references
}), which it may not.
cli-check-too-long = { $name } has { $lines } code lines, at most { $limit } are allowed.
cli-check-duplicate = { $name } copies { $original } ({ $path }:{ $line }), { $percent } % similar.
cli-check-unchecked = Skipped { $path }: { $code } { $message }
cli-check-summary = { $files ->
    [one] 1 file
   *[other] { $files } files
} checked, { $findings ->
    [one] 1 finding
   *[other] { $findings } findings
}.
cli-check-not-checked = Not checked yet: { $codes }.
cli-check-unmatched = No file matches { $path }. Check the path.
cli-cmd-open = Open CodeSkimr in the browser (the local server must be running)
cli-arg-no-browser = Do not start a browser; only create the launch file and print its path
cli-arg-print-url = With --no-browser: print the address with a one-time code (no launch file); the code is then in your terminal or log
cli-open-url-warning = Anyone who reads this address within { $seconds } s can sign in once. Keep it out of logs and shared screens.
cli-open-ready = Launch file: { $path } (valid for { $seconds } s, works once)
cli-cmd-export = Export a scanned project as one self-contained HTML file (without source code)
cli-arg-export-project = Id of the project, or the folder or Git URL it was scanned from
cli-arg-html = File that the HTML export is written to
cli-export-written = Export written to { $path } ({ $symbols } symbols, { $kilobytes } KiB). The source code is not included.
cli-cmd-credential = Store, replace or remove the credentials that log in to private Git repositories
cli-cmd-credential-set = Store a credential; the secret is read from the terminal (hidden) or from standard input
cli-cmd-credential-delete = Remove a stored credential
cli-cmd-credential-login = Sign in to GitHub or GitLab in the browser (OAuth device flow) and store the token
cli-arg-id = Name of the credential, for example work-github
cli-arg-user = User name that goes with the secret
cli-arg-provider = The platform to sign in to
cli-arg-client-id = Id of the OAuth application that is registered at the platform
cli-arg-host = Address of a self-managed installation, for example https://git.example.com
cli-arg-scan-name = Display name of the project; also renames a project that is registered already
cli-arg-scan-credential = Name of a stored credential that logs in to the Git source (see the credential command)
cli-credential-stored = Credential { $id } stored.
cli-credential-deleted = Credential { $id } removed.
cli-secret-prompt = Secret (not shown):
cli-credential-device-prompt = Open { $uri } and enter the code { $user_code }. Waiting for your approval …
cli-credential-unusable-hint = A credential only applies to a Git source, not to a folder, and its name must not be empty.
cli-scan-level2-checked = Level 2: { $language } checked ({ $files } files)
cli-scan-level2-partial = Level 2: { $language } checked ({ $files } files, { $stale } changed since the scan and stay at level 1)
cli-scan-level2-skipped = Level 2: { $language } skipped ({ $reason })
cli-scan-level2-failed-workspaces =
    Level 2: { $language } { $count ->
        [one] 1 workspace did not run through and stays
       *[other] { $count } workspaces did not run through and stay
    } at level 1
cli-scan-level2-outside-scan = Level 2: { $language } uses of code in folders the scan skips, not counted: { $count }
cli-level2-reason-not-permitted = this caller may not start indexers
cli-level2-reason-not-confirmed = not enabled and confirmed for this project
cli-level2-reason-unavailable = indexer or toolchain missing or too old
cli-level2-reason-no-project-file = the project folder lacks the project file the indexer needs (such as Cargo.toml or a .sln)
cli-level2-reason-stale = every file changed since the scan
cli-level2-reason-cancelled = scan cancelled
cli-level2-reason-failed = failed, { $code }: { $message }
cli-cmd-level2 = Allow or forbid level 2 (indexers) for a project
cli-cmd-level2-confirm = Allow the indexers to run for a project (asks you first)
cli-cmd-level2-revoke = Forbid the indexers for a project
cli-arg-level2-project = Id of the project, or the folder or Git URL it was scanned from
cli-arg-level2-from-ui = Set by the page in the browser: the window was opened for it
cli-level2-from-ui-notice = This window was opened by the CodeSkimr page in your browser. If you did not click it, close it.
cli-level2-from-ui-wait = Press Enter to close this window.
cli-level2-warning = Level 2 starts the build tools of this project (the compiler, the package manager and the indexer of the language) on this computer. They run the project's own code and scripts with your rights: they can read and change files of your user account and use the network. The indexers are not sandboxed. The confirmation is bound to the source, the location and the branch of the project, not to a commit: whatever the branch contains later is run under it. Only confirm for code you trust.
cli-level2-target = Project: { $project }
    Location: { $place }
cli-level2-target-branch = Branch: { $branch }
cli-level2-prompt = Type the name of the project ("{ $project }") to allow level 2 for it, anything else cancels:
cli-level2-confirmed = Level 2 is allowed for project { $project }. It runs with the next scan on the command line.
cli-level2-declined = Nothing changed: level 2 stays off for project { $project }.
cli-level2-revoked = Level 2 is forbidden again for project { $project }.
cli-scan-warning-ambiguous-reference = Warning RES-301: { $count } places match several symbols and are stored as probable.
cli-scan-warning-other = Warning { $code }: { $count } times.
cli-scan-warning-files-left-out = { $count ->
    [one] Warning { $code }: one file could not be analysed and was left out.
   *[other] Warning { $code }: { $count } files could not be analysed and were left out.
}
cli-scan-left-out-file = {"  "}{ $path } ({ $code }): missing in the result
cli-scan-left-out-file-kept = {"  "}{ $path } ({ $code }): shown as the previous scan saw it
cli-scan-left-out-more = { $count ->
    [one] {"  "}and one more file
   *[other] {"  "}and { $count } more files
}
cli-cmd-annotations = Notes, tasks and critical marks on symbols
cli-cmd-annotations-list = List the annotations of a project
cli-cmd-annotations-add = Add a critical mark, a note or a task to a symbol
cli-cmd-annotations-update = Change an annotation, or assign it to another symbol
cli-cmd-annotations-remove = Delete an annotation with its alerts
cli-cmd-alerts = Alerts for critical symbols that changed, were removed or lost their place
cli-cmd-alerts-list = List the alerts of a project
cli-cmd-alerts-dismiss = Dismiss one alert
cli-cmd-alerts-dismiss-all = Dismiss all open alerts of a filter; needs --yes
cli-arg-note-project = Id of the project, or the folder or Git URL it was scanned from
cli-arg-annotation = Id of the annotation
cli-arg-alert = Id of the alert
cli-arg-symbol = Qualified name of the symbol
cli-arg-symbol-kind = Kind of the symbol, such as class, method or namespace
cli-arg-param = Parameter type of the symbol, once per parameter, to tell overloads apart
cli-arg-kind = Kind of annotation: critical, note or task
cli-arg-status = Status of a task: open, in_progress or done
cli-arg-title = Title of a task
cli-arg-text = Text of the annotation, or the reason of a critical mark
cli-arg-due = Due date of a task, in seconds since the Unix epoch
cli-arg-assignee = Who a task is assigned to
cli-arg-clear-text = Remove the text
cli-arg-clear-due = Remove the due date
cli-arg-clear-assignee = Remove the assignee
cli-arg-limit = Rows to show; the configured page size if left out
cli-arg-offset = Rows to skip, to read the next page
cli-arg-alert-state = Only alerts in this state: open (default) or dismissed
cli-arg-alert-kinds = Only alerts of this kind: changed, removed or orphaned; can be repeated
cli-arg-yes = Confirm that all matching alerts are dismissed
cli-annotation-line = #{ $id } [{ $kind }] { $name }{ $parts ->
    [0] { "" }
   *[other] : { $summary }
}
cli-annotation-saved = Saved annotation #{ $id } on { $name }.
cli-annotation-removed = Deleted annotation #{ $id }.
cli-annotation-none = No annotations.
cli-annotations-page = Showing { $shown } of { $total }; continue with --offset { $next }.
cli-alert-line = #{ $id } [{ $state }/{ $kind }] { $name }{ $parts ->
    [0] { "" }
   *[other] : { $detail }
}
cli-alert-none = No alerts.
cli-alerts-page = Showing { $shown } of { $total } alerts, { $open } open; continue with --offset { $next }.
cli-alerts-dismissed = { $dismissed ->
    [one] One alert dismissed, { $open } still open.
   *[other] { $dismissed } alerts dismissed, { $open } still open.
}
cli-cmd-codes = List the error codes, or explain one
cli-arg-code = The code to explain, for example SRC-201 (default: list all codes)
cli-codes-line = { $code }  { $message }
cli-codes-explain =
    { $code }: { $message }
    { $description }
    Group { $group }, { $severity ->
        [warning] warning
        [info] information
       *[error] error
    }, { $retryable ->
        [yes] a retry can succeed
       *[no] a retry does not help
    }, since version { $since }.
cli-codes-retired = This code is retired: nothing raises it any more, and its number stays reserved.

cli-cmd-scope = Show or change which languages and folders a project covers
cli-arg-scope-project = Id of the project, or the folder or Git URL it was scanned from
cli-arg-without-language = Leave this language out of the scans (can be given more than once)
cli-arg-with-language = Take this language back in (can be given more than once)
cli-arg-without-folder = Leave this folder out of the scans, relative to the project folder (can be given more than once)
cli-arg-with-folder = Take this folder back in (can be given more than once)
cli-arg-scope-reset = Take everything back in before the other changes apply
cli-scope-heading = Scope of project { $project }:
cli-scope-language = { $state ->
    [covered] Covered
   *[left_out] Left out
}: language { $language } ({ $files } files)
cli-scope-folder = { $state ->
    [covered] Covered
   *[left_out] Left out
}: folder { $path } ({ $files } files)
cli-scope-more = { $count } more folders are not listed. Use --json for the whole answer.
cli-scope-abapgit = abapGit project: starting folder "{ $folder }", folder logic { $logic }, { $objects } objects in { $packages } packages, { $files } source files.
cli-scope-saved = The scope is saved. It applies from the next scan; the code stays one project.
cli-scope-unchanged = The scope stays as it was.
# The query commands symbols, usages, impact and context.
cli-cmd-symbols = Search symbols by name in the newest scan of a project (the answer of the MCP tool search_symbols)
cli-cmd-usages = List where a symbol is used, or what it uses (the answer of find_usages)
cli-cmd-impact = List what a change of a symbol can reach (the answer of impact_of_change)
cli-cmd-context = Give signature, callers, dependencies, affected files and notes of a symbol in one answer (the answer of get_context)
cli-cmd-unused = List the symbols for which no usage was found, with the reason for each (the answer of find_unused)
cli-cmd-rules = Judge the architecture rules of the project file against the newest scan; fails only for violations that the held state does not have (the answer of check_rules)
cli-query-rules-head = Rules: { $new } new, { $held } held as old burdens, { $resolved } resolved.
cli-query-rules-held = { $entries ->
    [one] State held on { $date }: 1 violation now counts as an old burden. Commit the file .codeskimr-baseline.toml.
   *[other] State held on { $date }: { $entries } violations now count as old burdens. Commit the file .codeskimr-baseline.toml.
}
cli-query-rules-none = The project file declares no architecture rules.
cli-query-rules-rule = { $rule }: { $new } new of { $total }
cli-query-rules-line = { $from } -> { $to }  { $path }:{ $line }:{ $column }  ({ $held ->
        [true] held
       *[other] NEW
    }, { $places ->
        [one] 1 place
       *[other] { $places } places
    })
cli-arg-hold = Hold the state instead of judging: every violation of now counts as an old burden; writes .codeskimr-baseline.toml next to the project file (folder projects only)
cli-query-rules-cycle-line = { $chain }  ({ $modules } modules, weakest edge { $weakest ->
        [one] 1 reference
       *[other] { $weakest } references
    } at { $places ->
        [one] 1 place
       *[other] { $places } places
    }; resolved over { $edges ->
        [one] 1 edge
       *[other] { $edges } edges
    }, { $held ->
        [true] held
       *[other] NEW
    })
cli-arg-query-text = Part of the symbol name; the characters must appear in this order
cli-arg-query-symbol = Id or qualified name of the symbol
cli-arg-query-project = Id of the project, or the folder or Git URL it was scanned from (default: the current folder)
cli-arg-kind-filter = Only symbols of this kind, such as class, function or constant
cli-arg-direction = incoming (who uses the symbol, default) or outgoing (what it uses)
cli-arg-depth = Steps the walk goes (at least 1; the configured default if left out)
cli-arg-include-tests = Count references from test code
cli-arg-budget = How much the answer holds: short, medium (default) or long
cli-arg-unused-kind = Only these symbols: all (default), types or functions
cli-arg-language-filter = Only symbols of this language, such as csharp or python
cli-arg-assessment = Only symbols with this assessment: likely, public or approximate
cli-arg-reason = Only symbols with this reason: private, internal, public or approximate
cli-cmd-preview-rename = Show what a new name for a symbol would hit: places in the code, name conflicts and text hits; changes nothing (the answer of preview_rename)
cli-arg-query-new-name = The new name: letters, digits and underscores, not starting with a digit
cli-rename-head = Rename preview: { $name } -> { $new }. CodeSkimr changes nothing.
cli-rename-sites = Places that would have to change: { $shown } in { $files } files ({ $verified } checked by the indexer)
cli-rename-declaration-line = { $path }:{ $line }:{ $column }  declaration
cli-rename-conflicts = Name conflicts ({ $shown } of { $total }):
cli-rename-conflict-line = { $kind } { $name }  { $path }  ({ $clash ->
        [true] same signature: the two clash
       *[other] different signature: maybe an overload
    })
cli-rename-texts = Text hits that CodeSkimr cannot attribute; please check them yourself ({ $shown } of { $total }):
cli-rename-text-line = { $path }:{ $line }  { $kind }  { $text }
cli-rename-not-searched = The project folder was not searched for text (no right to read source, or the project is not a folder). Strings, configuration and documentation are therefore missing.
cli-rename-public = The symbol is public. CodeSkimr does not see callers outside this project.
cli-rename-disclaimer = CodeSkimr does not say whether the code compiles afterwards. The compiler does.
cli-query-none = Nothing found.
cli-query-symbol-line = { $kind } { $name }  { $path }  (id { $id })
cli-query-usage-line = { $path }:{ $line }:{ $column }  { $kind }, { $confidence }, in { $name }
cli-query-impact-line = step { $depth }  { $kind } { $name }  { $path }
cli-query-unused-line = { $kind } { $name }  { $path }  ({ $assessment }, { $reason })
cli-query-unused-left-out = Left out by the exception list ({ $shown } of { $total }):
cli-query-unused-left-out-line = { $kind } { $name }  { $path }  because: { $text }
cli-query-cut = The answer was cut to the configured limit; use --limit, or --json for the totals.
cli-query-preliminary = Preliminary: some results were found by name resolution only and no indexer (level 2) has checked them.
cli-query-stale = Files changed since the last scan, so this answer may be out of date. Run codeskimr scan first.
cli-context-head = { $kind } { $name }  ({ $path }:{ $line })
cli-context-signature = Signature: { $text }
cli-context-layer = Layer: { $text }
cli-context-callers = Callers ({ $shown } of { $total }):
cli-context-dependencies = Depends on ({ $shown } of { $total }):
cli-context-dependency-line = { $count } x { $kind } { $name }  { $path }
cli-context-files = Files a change can reach ({ $shown } of { $total }):
cli-context-file = { $path }
cli-context-notes = Notes ({ $shown } of { $total }):
cli-context-note-line = { $kind }: { $text }
# The commands scans, compare and change (ADR 0097).
cli-cmd-scans = List the scans of a project that are kept, with time, kind, duration and changed files (the answer of the command scans.list)
cli-cmd-compare = Compare two scans of a project: the symbols that were added, removed, changed or moved, the new layer violations and cycles, and the critical symbols the difference touches
cli-cmd-change = Show what the difference between two scans hits: for every removed symbol and every symbol whose signature or visibility changed, its direct usages, and those that were not touched (the answer of the MCP tool impact_of_diff)
cli-arg-compare-project = Id of the project, or the folder or Git URL it was scanned from (default: the current folder)
cli-arg-compare-base = Id of the older scan (default: the scan before the target); `codeskimr scans` lists the ids
cli-arg-compare-target = Id of the newer scan (default: the newest scan)
cli-arg-markdown = Print the answer as Markdown for a comment on a pull request; cannot be combined with --json
cli-cmd-modules = Show the modules (namespaces) of the project: references in and out, stability, the cycles between modules with the dependencies that end them, the dependencies against the layering and the modules that stand out (the answer of get_dependencies with level module)
cli-arg-modules-depth = Merge the namespaces into their first N segments (all of them if left out)
cli-arg-modules-from = The module that depends on the other; with --to, the places of that dependency are listed instead of the modules
cli-arg-modules-to = The module that is depended on (with --from)
cli-query-modules-none = No modules found.
cli-query-modules-places-head = Places of { $from } -> { $to }: { $shown } of { $total } shown.
cli-query-modules-place = { $path }:{ $line }  { $from } -> { $to }
cli-query-modules-places-none = No places of { $from } -> { $to } found.
cli-query-modules-head = Modules: { $shown } of { $total } shown, the most referenced first. in: places in other modules that use it; out: places in it that use other modules; stability = out / (in + out).
cli-query-modules-line = { $name }{ $layer }  in { $incoming }, out { $outgoing }, stability { $stability }{ $flags }
cli-query-modules-layer = [{ $name }]
cli-query-modules-stability = { $whole }.{ $hundredths }
cli-query-modules-stability-none = none
cli-query-modules-flag-cycle = in a cycle
cli-query-modules-flag-violations = { $count ->
    [one] 1 dependency against the layering
   *[other] { $count } dependencies against the layering
}
cli-query-modules-cycles = Cycles: { $shown }
cli-query-modules-cycle = { $modules } modules in a ring: { $chain }
cli-query-modules-resolve = Ending it takes away { $from } -> { $to } ({ $count ->
    [one] 1 place
   *[other] { $count } places
})
cli-query-modules-violations = Against the layering: { $shown }
cli-query-modules-violation = { $from } ({ $from_layer }) -> { $to } ({ $to_layer }): { $count ->
    [one] 1 place
   *[other] { $count } places
}
cli-query-modules-risks = Stands out: { $shown }
cli-query-modules-risk = { $name } (stability { $stability }) is used in { $incoming } places and depends on { $other } (stability { $otherStability }), which changes more often.
