# Projects screen.

projects-title = Projects
projects-summary =
    { $projects ->
        [one] { $projects } project
       *[other] { $projects } projects
    } · { $types ->
        [one] { $types } type
       *[other] { $types } types
    }
projects-new = New project
projects-new-title = New project
projects-field-source = Source
projects-field-branch = Branch
projects-field-name = Display name
projects-name-hint = Just a label: the folder and the number stay the same. Leave it empty to use the suggested name.
projects-rename = Rename { $name }
projects-rename-button = Rename
projects-rename-field = New display name for { $name }
projects-rename-save = Save
projects-rename-saving = Saving …
projects-rename-cancel = Cancel
projects-add-and-scan = Add and scan
projects-column-project = Project
projects-column-source = Source
projects-column-languages = Languages
projects-column-types = Types
projects-column-loc = LOC
projects-column-last-scan = Last scan
projects-column-status = Status
projects-column-actions = Actions
projects-hint-keys = Enter opens · R shows the scan · N new project

projects-source-placeholder = Paste a folder path or Git URL – or drop a folder here
projects-source-folder = Folder
projects-source-git = Git
projects-status-current = current
projects-status-running = scan running
projects-status-unscanned = not scanned
projects-scan-full = full
projects-scan-incremental = incremental
projects-scan-seconds = { $seconds } s
projects-scan-time = { $mode } · { $seconds }
projects-adding = Scanning …
projects-drop-without-path = The browser does not reveal the path of a dropped folder. Paste the path into the field instead.
projects-load-failed = The project list could not be loaded.
projects-open = Open { $name }
projects-list-label = Projects
projects-language-mix = { $name } { $percent } %
projects-remove-button = Remove
projects-remove = Remove project { $name }
projects-remove-title = Remove project "{ $name }"?
projects-remove-effect-data = The project file is deleted, with all scans, results, notes and tasks of the project.
projects-remove-effect-data-size = The project file ({ $size }) is deleted, with all scans, results, notes and tasks of the project.
projects-remove-effect-files = The files of the project stay untouched.
projects-remove-effect-credentials = Stored logins stay in the key store.
projects-remove-keys = Esc cancels
projects-dialog-close = Close
projects-new-keys = Esc closes · Enter adds
projects-field-address = Repository address
projects-field-optional = optional
projects-remove-confirm = Remove for good
projects-remove-cancel = Cancel
projects-removing = Removing …
projects-removal-interrupted = Removal interrupted
projects-never = never
projects-header = { $summary } · { $hints }

projects-tools-title = Tools for exact usages (level 2): { $name }
projects-tools-missing-count =
    { $count ->
        [one] 1 tool missing
       *[other] { $count } tools missing
    }
projects-tools-warning = Without these tools, usages of the languages named stay unchecked.
projects-tools-state-missing = missing
projects-tools-state-not-runnable = found, but it does not start: { $found }
projects-tools-state-not-runnable-silent = found, but it does not start
projects-tools-state-too-old = too old: { $found } installed, { $required } required
projects-tools-state-too-new = too new: { $found } installed, more than the project allows (it names { $required })
projects-tools-state-unknown = version not readable (required: { $required })
projects-tools-for = for { $languages }
projects-tools-purpose = What it is for: { $text }
projects-tools-install = How to get it: { $text }
projects-tools-ready-languages = Level 2 can run for: { $languages }
projects-tools-all-ready = All tools are there.
projects-tools-no-level2 = No language of this project has level 2 yet.
projects-tools-unchecked = The tools have not been checked yet.
projects-tools-check-again = Check again
projects-tools-checking = Checking …
projects-tools-checked = Last checked: { $when }

projects-credential-label = Login
projects-credential-none = None (the system's Git login)
projects-credential-new = Store a new login
projects-credential-name = Name
projects-credential-user = User name
projects-credential-secret = Token or password
projects-credential-save = Store login
projects-credential-saving = Storing …
projects-credential-saved = Stored { $name }
projects-credential-hint = The secret goes to the keystore of this computer and is never shown again.

projects-level2-column = Level 2
projects-level2-switch-label = Level 2 for { $name }
projects-level2-state-on = on
projects-level2-state-off = off
projects-level2-state-not-enabled = not enabled
projects-level2-state-locked = locked
projects-level2-state-unknown = unknown
projects-level2-title = Level 2 for exact usages: { $name }
projects-level2-explain-on = Level 2 is on. The next scan checks usages exactly, as far as the tools for the languages of the project are installed.
projects-level2-explain-off = Level 2 is off, scans stay at level 1. To switch it on, you confirm in a console window that build tools of this project may run.
projects-level2-explain-not-enabled = Level 2 is not enabled on this computer. CodeSkimr never changes this setting itself. Add these two lines to your configuration file config.toml, save it and restart CodeSkimr:
projects-level2-config-file = File: in your config.toml in the data folder
projects-level2-explain-locked = Level 2 is locked. A team server does not run build tools.
projects-level2-explain-unknown = The state of level 2 could not be read.
projects-level2-check-again = Read the state again
projects-level2-switch-on = Switch on
projects-level2-switch-off = Switch off
projects-level2-opening = Opening the console window …
projects-level2-waiting-title = Finish the confirmation in the console window
projects-level2-waiting-text = A window was opened at { $time }. Read the warning there and type the project name to confirm. This page asks again and shows the result here. If you did not click, close that window.
projects-level2-waiting-state = Waiting for the confirmation …
projects-level2-waiting-cancel = Cancel waiting
projects-level2-waiting-pause = After a window was opened, a pause of { $seconds } seconds applies. If you close the window and click again too early, the server answers with a pause notice (API-504). Then wait briefly and try again.
projects-level2-cancelled = Waiting ended. Nothing was confirmed and level 2 stays off. You can switch on again.
projects-level2-timed-out = Not confirmed: the confirmation did not arrive in time. Close the console window if it is still open. You can try again by switching on.
projects-level2-copy-title = Finish the confirmation in a terminal
projects-level2-copy-text = This computer cannot open a console window from here. Run this command in a terminal and type the project name when asked:
projects-level2-copy = Copy
projects-level2-copied = Copied
projects-level2-command-label = Command to confirm level 2 for { $name }
projects-level2-revoke-ask = Switch off level 2 for { $name }?
projects-level2-revoke-text = Checked results stay until the next scan. The next scan runs level 1 only.
projects-level2-revoke-yes = Yes, switch off
projects-level2-revoke-no = Cancel
projects-level2-revoking = Switching off …
projects-level2-revoked = Level 2 is off. Checked results stay until the next scan. The next scan runs level 1 only.
projects-level2-rescan = Scan again
projects-level2-retry-wait = Wait { $seconds } seconds, then switch on again.
projects-credential-settings = Manage logins in the settings
projects-field-source-kind = Source
projects-source-placeholder-folder = Paste a folder path – or drop a folder here
projects-source-placeholder-git = Paste the address of a Git repository
projects-report-origin-terminal = terminal
projects-report-origin-browser = browser
projects-report-origin-watch = watch mode
projects-report-origin-mcp = MCP
projects-report-line = Stage { $level } ({ $origin })
projects-report-counts = { $checked } checked, { $skipped } skipped, { $failed } failed
projects-report-stale = { $count } files changed since
projects-report-failed-workspaces =
    { $count ->
        [one] 1 workspace did not run through and stays at level 1
       *[other] { $count } workspaces did not run through and stay at level 1
    }
projects-report-failed-code = Stage 2 failed ({ $code })
projects-report-title = Last stage 2 run: { $time } ({ $origin })
projects-report-none = No stage 2 run is stored for this project yet.
projects-report-checked = { $language }: checked, { $files } files
projects-report-checked-stale = { $language }: checked, { $files } files, { $stale } changed since
projects-report-skipped = { $language }: skipped, { $reason }
projects-report-failed = { $language }: failed ({ $code }) – { $text }
projects-report-group-failed = Failed
projects-report-group-failed-count = { $count ->
    [one] 1 language needs attention
   *[other] { $count } languages need attention
  }
projects-report-group-skipped = Skipped
projects-report-group-skipped-count = { $count ->
    [one] 1 language, a requirement is missing
   *[other] { $count } languages, a requirement is missing
  }
projects-report-group-checked = Checked
projects-report-group-checked-count = { $count ->
    [one] 1 language
   *[other] { $count } languages
  }
projects-report-group-no-level2 = No stage 2 available
projects-report-group-no-level2-count = { $count ->
    [one] 1 language, there is no exact check for it yet
   *[other] { $count } languages, there is no exact check for them yet
  }
projects-report-no-level2-note = Their references stay approximations from stage 1.
projects-report-switched-off = Stage 2 is off or not confirmed for this project. Switch it on above and confirm it to check { $count ->
    [one] 1 language
   *[other] { $count } languages
  }: { $languages }.
projects-report-chip = { $language } · { $files } files
projects-report-chip-failed-workspaces =
    { $count ->
        [one] 1 workspace not run through
       *[other] { $count } workspaces not run through
    }
projects-report-chip-stale = { $language } · { $files } files · { $stale } outdated
projects-report-todo = What to do: { $text }
projects-report-action-rescan = Check again
projects-report-action-tools = Go to Tools
projects-report-todo-not-permitted = Start stage 2 from the browser or the terminal, not in watch mode or through MCP.
projects-report-todo-not-confirmed = Switch stage 2 on above and confirm it, then scan again.
projects-report-todo-tools-unavailable = Install or update the tool (see the Tools tab).
projects-report-todo-no-project-file = If the file lies in a subfolder, add that folder as a project of its own.
projects-report-todo-stale = Scan again.
projects-report-todo-retry = Check again.
projects-report-todo-later = Scan again later.
projects-report-todo-RES-504 = Raise the memory limit (level2.max_memory_mb, or level2.indexer_memory_mb for this indexer alone, in the configuration), then check again.
projects-report-todo-RES-502 = Raise the time limit (level2.timeout_seconds in the configuration), then check again.
projects-report-todo-RES-503 = Raise the output limit (level2.max_output_bytes in the configuration), then check again.
projects-report-todo-RES-601 = Look at the log (set the environment variable CODESKIMR_LOG=info for the scan), then check again.
projects-report-failed-note = The last run failed. Results of an earlier run may remain for files that did not change.
projects-report-reason-not-permitted = this way of scanning may not run indexers
projects-report-reason-not-confirmed = level 2 is not confirmed for this project
projects-report-reason-tools-unavailable = the indexer or a tool it needs is missing, too old or does not start
projects-report-reason-no-project-file = the project folder lacks a project file that the indexer needs
projects-report-reason-no-project-file-rust = no Cargo.toml found in the project folder, above it or in a folder of the Rust files
projects-report-reason-no-project-file-csharp = no .sln, .slnx or .csproj found in the project folder, above it or in a folder of the C# files
projects-report-reason-stale = every file changed since the run
projects-report-reason-cancelled = cancelled
projects-report-reason-busy = not started again because another stage 2 run was going on; earlier checked results stay
projects-report-reason-paused = not started again because of the pause after the last run; earlier checked results stay
projects-level2-running = Stage 2 running ({ $language })
projects-level2-running-importing = Stage 2 importing results ({ $language })
projects-level2-running-unknown = Stage 2 running
projects-level2-cancel = Cancel stage 2
projects-level2-cancel-note = Cancelling keeps the result of stage 1.
projects-level2-build-note = Stage 2 runs the build tools of the project, for a Git project after fetching new code. It can take minutes.
projects-level2-stage1-only = Stage 1 only
projects-level2-stage1-only-hint = A quick scan without build tools.
projects-level2-full-rescan = Full scan
projects-level2-full-rescan-hint = Reads every file again, not only the changes (like "codeskimr scan" in the console).
projects-level2-scan-title = Scan
projects-level2-finished = Stage 2 ended: { $checked } checked, { $skipped } skipped, { $failed } failed.
projects-level2-not-started-not-permitted = Stage 2 did not start: this program may not run indexers here. Stage 1 is stored.
projects-level2-not-started-not-confirmed = Stage 2 did not start: level 2 is not confirmed for this project (confirm it in the "Exact check (stage 2)" tab). Switch it on above, then scan again. Stage 1 is stored.
projects-level2-not-started-busy = Stage 2 did not start: another stage 2 run is going on. Stage 1 is stored; scan again when that run has ended.
projects-level2-not-started-paused = Stage 2 did not start: after a run there is a pause. Stage 1 is stored; scan again in a minute.
projects-browse = Browse …
projects-browse-waiting-title = Folder dialog open
projects-browse-waiting-text = Choose a folder in the window of the operating system. The window may be behind this one. This page waits until you choose or cancel.
projects-browse-give-up = Stop waiting
projects-browse-done = Folder taken over. Check the path and click "Add and scan".

projects-new-short = New
projects-filter-label = Filter projects
projects-filter-placeholder = Filter projects …
projects-filter-none = No project matches.
projects-tabs-label = Sections of the project
projects-tab-overview = Overview
projects-tab-scan = Scan
projects-tab-level2 = Exact check (stage 2)
projects-tab-tools = Tools
projects-tile-types = Types
projects-tile-lines = Lines
projects-tile-files = Files
projects-tile-last-scan = Last scan
projects-state-level2-failed = Exact check failed
projects-note-level2-failed = Exact check (stage 2) failed for { $languages } ({ $code }). The structure is complete; usages in these languages are only approximate. Details are in the tab “Exact check (stage 2)”.
projects-languages-title = Languages
projects-column-share = Share
projects-column-accuracy = Accuracy
projects-accuracy-exact = Exact (stage 2)
projects-accuracy-approximate = Approximation (stage 1)
projects-accuracy-failed = Stage 2 failed
projects-card-last-scan = Last scan
projects-card-tools = Tools
projects-card-tools-ok = all there
projects-card-database = Database
projects-card-left-out = { $count } left out
projects-db-state-ok = present
projects-db-state-missing = missing
projects-db-state-removing = being removed
projects-db-state-unreadable = File not readable
projects-db-size = Size { $size }
projects-db-unavailable = Not available yet.
projects-db-copy = Copy path
projects-db-copied = Copied
projects-source-line = { $parts }
projects-scan-start-hint = Start a scan with “Scan again”; its progress shows up here.
projects-language-percent = { $percent } %
projects-open-explorer = Open in explorer
projects-list-progress = Scan progress of { $name }
