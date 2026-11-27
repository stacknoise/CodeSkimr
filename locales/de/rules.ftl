# Die Seite „Regeln“: Architekturregeln und ihre Verstöße (ADR 0104).

nav-rules = Regeln
rules-title = Regeln
rules-subtitle = aus .codeskimr.toml · geprüft gegen den letzten Scan
rules-no-project = Wählen Sie zuerst ein Projekt, dessen Regeln Sie sehen möchten.
rules-not-scanned = Das Projekt wurde noch nicht gescannt, deshalb kann keine Regel beurteilt werden. Scannen Sie es zuerst.
rules-load-failed = Die Regeln des Projekts konnten nicht gelesen werden.
rules-none = Die Projektdatei enthält noch keine Regeln. Schreiben Sie sie in die Tabelle [rules] der .codeskimr.toml des Projekts; siehe Handbuch.
rules-kept-count = { $count } eingehalten
rules-kept-label = Regeln
rules-failing-count = { $count } lassen die CI fehlschlagen
rules-failing-label = neue Verstöße
rules-held-count = { $count } festgehalten am { $date }
rules-held-label = Altlasten
rules-held-none = Kein Stand festgehalten
rules-held-none-label = jeder Verstoß zählt
rules-resolved-count = { $count } behoben
rules-resolved-label = seit dem Festhalten
rules-kind-new = neue Regelart
rules-forbid-title = Verbot: { $from } darf { $to } nicht nutzen
rules-forbid-spec = forbid = { "{" } from = "{ $from }", to = "{ $to }" { "}" }
rules-public-api-title = Nur über die öffentliche Schnittstelle
rules-public-api-spec = { $internal } nur innerhalb von { $module }
rules-state-kept = eingehalten
rules-badge-new = { $count } neu
rules-badge-old = { $count ->
    [one] 1 Altlast
   *[other] { $count } Altlasten
}
rules-violation-old = Altlast
rules-violation-new = neu
rules-places = { $count ->
    [one] 1 Stelle
   *[other] { $count } Stellen
}
rules-more = … { $count } weitere
rules-show-all = Alle zeigen
rules-show-less = Weniger zeigen
rules-hold = Stand festhalten
rules-hold-hint = „Stand festhalten“: Vorhandene Verstöße gelten als Altlast. Die CI schlägt nur bei neuen fehl.
rules-hold-git = Nur für Ordnerprojekte: Ein Git-Projekt hat keine Projektdatei, neben die die Datei käme.
rules-hold-done = { $count ->
    [one] Stand festgehalten am { $date }: 1 Verstoß gilt jetzt als Altlast. Die Datei .codeskimr-baseline.toml gehört ins Repository.
   *[other] Stand festgehalten am { $date }: { $count } Verstöße gelten jetzt als Altlast. Die Datei .codeskimr-baseline.toml gehört ins Repository.
}
rules-hold-failed = Der Stand konnte nicht festgehalten werden.
rules-open = .codeskimr.toml öffnen
rules-open-hint = Öffnet die Projektdatei mit dem Programm, das Ihr System dafür vorsieht. Fehlt sie, wird sie leer angelegt.
rules-open-git = Nur für Ordnerprojekte: Ein Git-Projekt hat keine Projektdatei.
rules-open-created = Die Datei { $file } gab es noch nicht und wurde leer angelegt.
rules-open-failed = Die Projektdatei konnte nicht geöffnet werden.
rules-layers-title = Schichten
rules-layers-allowed = { $layer } → { $uses }
rules-layers-nothing = nichts
rules-cycles-title = Keine Zyklen zwischen Modulen
rules-cycles-spec = cycles = "deny"
rules-ring-advice = Auflösbar über { $edges ->
    [one] 1 Kante
   *[other] { $edges } Kanten
} mit { $places ->
    [one] 1 Stelle
   *[other] { $places } Stellen
}
rules-ring-show = Im Graphen zeigen
rules-add = Regel hinzufügen
rules-add-git = Nur für Ordnerprojekte: Ein Git-Projekt hat keine Projektdatei, in die die Regeln kämen.
rules-add-forbid = Verbot
rules-add-forbid-hint = Modul A darf Modul B nicht nutzen
rules-add-public-api = Nur über die öffentliche Schnittstelle
rules-add-public-api-hint = Das Innere eines Moduls bleibt geschlossen
rules-add-cycles = Zyklen verbieten
rules-add-cycles-hint = Ein Schalter, keine neue Karte
rules-add-layers = Schichten
rules-add-layers-hint = Was jede Schicht nutzen darf
rules-edit = Regel ändern
rules-delete = Regel löschen
rules-dialog-forbid-new = Verbot hinzufügen
rules-dialog-forbid-edit = Verbot ändern
rules-dialog-api-new = Regel hinzufügen: nur über die öffentliche Schnittstelle
rules-dialog-api-edit = Regel ändern: nur über die öffentliche Schnittstelle
rules-dialog-layers = Schichten: wer darf wen nutzen
rules-dialog-delete = Regel löschen?
rules-dialog-delete-body = Die Regel „{ $rule }“ wird aus der .codeskimr.toml entfernt. Ihre Verstöße werden nicht mehr gemeldet.
rules-field-from = Dieses Modul …
rules-field-to = … darf dieses Modul nicht nutzen
rules-field-reason = Grund (optional, wird auf der Seite gezeigt)
rules-field-module = Modul
rules-field-internal = Inneres (ein Präfix pro Zeile, jeweils unterhalb des Moduls)
rules-field-hint = Aus den Namensräumen des Scans ({ $count }); ein eigenes Präfix kann getippt werden.
rules-layers-help = Die Zeile nutzt die Spalten. Ohne „Limit“ darf eine Schicht alles nutzen; mit Limit nur die angehakten Schichten (keine, wenn nichts angehakt ist). Namen und Namensräume der Schichten bleiben im Schicht-Editor.
rules-layers-limit = Limit
rules-layers-uses = darf nutzen →
rules-layers-none = Das Projekt hat noch keine Schichten. Lege sie zuerst im Schicht-Editor an.
rules-preview-checking = Prüfe gegen den letzten Scan …
rules-preview-none = Mit dem letzten Scan gäbe es keinen Verstoß.
rules-preview-some = { $count ->
    [one] Mit dem letzten Scan wäre das 1 Verstoß ({ $old } davon schon festgehalten).
   *[other] Mit dem letzten Scan wären das { $count } Verstöße ({ $old } davon schon festgehalten).
}
rules-preview-unscanned = Das Projekt hat noch keinen Scan, darum gibt es keine Vorschau.
rules-preview-invalid = Das ist noch keine Regel: Prüfe die Namen.
rules-promise = Schreibt in die .codeskimr.toml. Übrige Tabellen und Kommentare bleiben; Kommentare innerhalb der Regel-Tabellen werden neu geschrieben.
rules-cancel = Abbrechen
rules-save = Speichern
rules-saving = Speichere …
rules-delete-confirm = Löschen
rules-save-failed = Die Regeln konnten nicht gespeichert werden.
rules-toml-label = Text, der in die Datei geschrieben wird
