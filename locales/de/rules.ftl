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
