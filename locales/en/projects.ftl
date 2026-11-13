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
