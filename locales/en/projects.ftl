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
projects-tools-state-too-old = too old: { $found } installed, { $required } required
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
