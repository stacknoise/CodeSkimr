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
rules-state-broken = verletzt
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
rules-cycles-pending = Zyklen zwischen Modulen sind in der Projektdatei eingeschaltet. Die Prüfung folgt in einer späteren Version.
