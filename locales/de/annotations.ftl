# Anmerkungen an Symbolen: Markierung "systemkritisch", Notizen und Aufgaben (ADR 0068).

annotations-critical-label = Kritisch
annotations-critical-initial = K
annotations-critical-system = Systemkritisch
annotations-panel-label = Seitenleiste
annotations-tabs-label = Anmerkungen des Symbols
annotations-tab-info = Info
annotations-tab-critical = Kritisch
annotations-tab-notes = Notizen
annotations-tab-tasks = Aufgaben
annotations-column = Anmerkungen
annotations-notes-count = { $count ->
    [one] 1 Notiz
   *[other] { $count } Notizen
}
annotations-tasks-count = { $done } von { $total } Aufgaben erledigt
annotations-critical-below = { $count ->
    [one] 1 kritisches Symbol darunter
   *[other] { $count } kritische Symbole darunter
}
annotations-show-text = Anmerkungen einblenden
annotations-inline-note = Notiz:
annotations-inline-task = Aufgabe:
annotations-inline-due = fällig { $date }
annotations-by = { $author } · { $date }
annotations-add-note = Notiz hinzufügen
annotations-add-task = Aufgabe hinzufügen
annotations-note-label = Text der Notiz
annotations-note-hint = Notizen werden als einfacher Text auf diesem Rechner gespeichert. Keine Passwörter oder Schlüssel hineinschreiben.
annotations-task-title = Titel
annotations-task-text = Beschreibung (optional)
annotations-task-due = Fällig am
annotations-task-assignee = Zuständig
annotations-task-status = Status
annotations-tasks-summary = Aufgaben ({ $done } von { $total } erledigt)
annotations-task-done-toggle = Erledigt: { $title }
annotations-task-due-line = fällig { $date }
annotations-task-late = überfällig seit { $date }
annotations-task-no-due = ohne Termin
annotations-task-assigned = { $name }
annotations-status-open = Offen
annotations-status-in-progress = In Arbeit
annotations-status-done = Erledigt
annotations-save = Speichern
annotations-cancel = Abbrechen
annotations-edit = Bearbeiten
annotations-delete = Löschen
annotations-delete-confirm = Endgültig löschen?
annotations-empty-notes = An diesem Symbol gibt es noch keine Notizen.
annotations-empty-tasks = An diesem Symbol gibt es noch keine Aufgaben.
annotations-without-target = Das Symbol ist im neuesten Scan nicht mehr vorhanden.
annotations-loading = Anmerkungen werden gelesen …
annotations-load-failed = Die Anmerkungen konnten nicht gelesen werden.
annotations-save-failed = Die Änderung konnte nicht gespeichert werden.
annotations-critical-on = Dieses Symbol ist als systemkritisch markiert.
annotations-critical-off = Dieses Symbol ist nicht als systemkritisch markiert.
annotations-critical-explain = Eine Änderung an einem kritischen Symbol löst nach dem nächsten Scan einen Alarm aus.
annotations-critical-reason = Grund (optional)
annotations-mark = Als kritisch markieren
annotations-unmark = Markierung entfernen
annotations-notice-marked = Als systemkritisch markiert.
annotations-notice-unmarked = Markierung entfernt.
annotations-keys-hint = Tasten: K kritisch, N Notiz, T Aufgabe
annotations-info-qualified = Qualifizierter Name
annotations-info-kind = Art
annotations-info-location = Ort
annotations-info-signature = Signatur
annotations-info-layer = Schicht
annotations-no-symbol = Wähle einen Typ oder ein Member, um seine Anmerkungen zu sehen.
annotations-command-critical = Auswahl als kritisch markieren oder Markierung entfernen
annotations-command-note = Neue Notiz zur Auswahl
annotations-command-task = Neue Aufgabe zur Auswahl
annotations-legend = Symbole: Schild = kritisch, Blatt = Notizen, Haken = Aufgaben (erledigt von allen)
