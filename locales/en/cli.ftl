# Command line interface.

cli-error-line = error[{ $code }]: { $message }
cli-scan-started = Scanning { $path } …
cli-scan-finished = Scanned { $files } files in { $seconds } s.
cli-unknown-command = Unknown command: { $command }
cli-help-hint = Run with --help to see all commands.
cli-codes-not-found = No error code with the id { $id } exists.
cli-scan-details = { $symbols } symbols, { $references } references, snapshot { $snapshot }.
cli-about = CodeSkimr turns code repositories into navigable documentation.
cli-cmd-scan = Scan a folder or a Git URL and print a summary
cli-cmd-mcp = Serve the MCP protocol over standard input and output
cli-cmd-serve = Start the local server without opening a browser
cli-serve-listening = Listening on { $url }. Stop with Ctrl+C.
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
cli-cmd-check = Check files against the code rules (length, duplicates) and exit with 1 if there are findings
cli-arg-paths = Files or folders to check, relative to the current folder (default: everything)
cli-check-finding = { $path }:{ $line }: { $code } { $message }
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
cli-arg-scan-credential = Name of a stored credential that logs in to the Git source (see the credential command)
cli-credential-stored = Credential { $id } stored.
cli-credential-deleted = Credential { $id } removed.
cli-secret-prompt = Secret (not shown):
cli-credential-device-prompt = Open { $uri } and enter the code { $user_code }. Waiting for your approval …
cli-credential-unusable-hint = A credential only applies to a Git source, not to a folder, and its name must not be empty.
