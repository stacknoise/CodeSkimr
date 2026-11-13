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
