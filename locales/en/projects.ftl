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
projects-add-and-scan = Add and scan
projects-column-project = Project
projects-column-source = Source
projects-column-languages = Languages
projects-column-types = Types
projects-column-loc = LOC
projects-column-last-scan = Last scan
projects-column-status = Status
projects-hint-keys = Enter opens · R rescans · N new project

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
projects-rescan = Scan { $name } again
projects-adding = Scanning …
projects-drop-without-path = The browser does not reveal the path of a dropped folder. Paste the path into the field instead.
projects-load-failed = The project list could not be loaded.
projects-open = Open { $name }
projects-list-label = Projects
projects-language-mix = { $name } { $percent } %
projects-remove-button = Remove
projects-remove = Remove project { $name }
projects-remove-question = Remove { $name } from the app? Scans, results and notes of the project are deleted. Its files stay untouched.
projects-remove-confirm = Remove for good
projects-remove-cancel = Cancel
projects-remove-ask = Confirm removal
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
projects-browse = Browse …
projects-browse-waiting-title = Folder dialog open
projects-browse-waiting-text = Choose a folder in the window of the operating system. The window may be behind this one. This page waits until you choose or cancel.
projects-browse-give-up = Stop waiting
projects-browse-done = Folder taken over. Check the path and click "Add and scan".
