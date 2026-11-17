# Annotations on symbols: system-critical marks, notes and tasks (ADR 0068).

annotations-critical-label = Critical
annotations-critical-initial = C
annotations-critical-system = System critical
annotations-panel-label = Side panel
annotations-tabs-label = Annotations of the symbol
annotations-tab-info = Info
annotations-tab-critical = Critical
annotations-tab-notes = Notes
annotations-tab-tasks = Tasks
annotations-column = Annotations
annotations-notes-count = { $count ->
    [one] 1 note
   *[other] { $count } notes
}
annotations-tasks-count = { $done } of { $total } tasks done
annotations-critical-below = { $count ->
    [one] 1 critical symbol below
   *[other] { $count } critical symbols below
}
annotations-show-text = Show annotations
annotations-inline-note = Note:
annotations-inline-task = Task:
annotations-inline-due = due { $date }
annotations-by = { $author } · { $date }
annotations-add-note = Add note
annotations-add-task = Add task
annotations-note-label = Text of the note
annotations-note-hint = Notes are stored in plain text on this computer. Do not write passwords or keys here.
annotations-task-title = Title
annotations-task-text = Description (optional)
annotations-task-due = Due date
annotations-task-assignee = Assignee
annotations-task-status = Status
annotations-tasks-summary = Tasks ({ $done } of { $total } done)
annotations-task-done-toggle = Done: { $title }
annotations-task-due-line = due { $date }
annotations-task-late = overdue since { $date }
annotations-task-no-due = no due date
annotations-task-assigned = { $name }
annotations-status-open = Open
annotations-status-in-progress = In progress
annotations-status-done = Done
annotations-save = Save
annotations-cancel = Cancel
annotations-edit = Edit
annotations-delete = Delete
annotations-delete-confirm = Delete for good?
annotations-empty-notes = No notes on this symbol yet.
annotations-empty-tasks = No tasks on this symbol yet.
annotations-without-target = The symbol is gone from the newest scan.
annotations-loading = Reading the annotations …
annotations-load-failed = The annotations could not be read.
annotations-save-failed = The change could not be saved.
annotations-critical-on = This symbol is marked system critical.
annotations-critical-off = This symbol is not marked as system critical.
annotations-critical-explain = A change to a critical symbol raises an alert after the next scan.
annotations-critical-reason = Reason (optional)
annotations-mark = Mark as critical
annotations-unmark = Remove mark
annotations-notice-marked = Marked as system critical.
annotations-notice-unmarked = Mark removed.
annotations-keys-hint = Keys: K critical, N note, T task
annotations-info-qualified = Qualified name
annotations-info-kind = Kind
annotations-info-location = Location
annotations-info-signature = Signature
annotations-info-layer = Layer
annotations-no-symbol = Select a type or a member to see its annotations.
annotations-command-critical = Mark selection as critical or take the mark off
annotations-command-note = New note on the selection
annotations-command-task = New task on the selection
annotations-legend = Symbols: shield = critical, document = notes, tick = tasks (done of all)
