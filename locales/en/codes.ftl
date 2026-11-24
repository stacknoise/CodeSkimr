# The list of error codes to look up in the UI; the counterpart of `codeskimr codes`.

codes-title = Error codes
codes-subtitle = { $count ->
    [one] { $count } code, as in `codeskimr codes`
   *[other] { $count } codes, as in `codeskimr codes`
}
codes-search-label = Search error codes
codes-search-placeholder = Search by code or text, such as SRC-201
codes-severity-all = All severities
codes-severity-error = Error
codes-severity-warning = Warning
codes-severity-info = Notice
codes-group-label = Group
codes-group-all = All groups
codes-col-code = Code
codes-col-message = Message
codes-col-group = Group
codes-col-severity = Severity
codes-col-retry = Retry
codes-col-since = Since
codes-retry-yes = Can help
codes-retry-no = No
codes-description = Developer description (English): { $description }
codes-facts = Group { $group }, { $severity }, retry: { $retry }, since version { $since }.
codes-retired = This code is retired: nothing raises it any more, the number stays reserved.
codes-empty = No code matches.
codes-note-retired = Retired codes (ADR 0078) are hidden. Type the full number of one to see it.
codes-back = Back to settings
settings-codes-title = Error codes
settings-codes-note = Look up every error code with its explanation, as `codeskimr codes` lists them.
settings-codes-action = Show error codes
