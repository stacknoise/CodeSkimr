# Die Ansicht "Änderung": was der Unterschied zwischen zwei Scans trifft (ADR 0097, Mocks 29 und erw-auswirkung).

nav-change = Änderung
change-view-title = Änderung
change-view-subtitle = Was trifft diese Änderung?
change-view-no-project = Zuerst ein Projekt wählen, dessen Scans verglichen werden sollen.
change-view-load-failed = Der Vergleich konnte nicht gelesen werden.
change-view-reading = Die Scans werden verglichen …
change-view-too-few = Das Projekt hat weniger als zwei Scans, also gibt es noch nichts zu vergleichen.
change-view-tabs-label = Ansichten der Änderung
change-view-tab-impact = Auswirkung
change-view-tab-compare = Vergleich
change-view-panel-label = Der Vergleich zweier Scans
change-view-compared = Vergleich

change-view-base = Basis (vorher)
change-view-target = Ziel (nachher)
change-view-run = Vergleichen
change-view-scan-option = Scan vom { $time }
change-view-scan-current = Aktueller Scan

change-view-copy = Als Markdown kopieren
change-view-copy-done = Kopiert
change-view-copy-failed = Nicht kopiert

change-view-tile-symbols = geänderte Symbole
change-view-tile-public = davon öffentlich
change-view-tile-usages = betroffene Verwendungen
change-view-tile-usages-certain = { $count } sicher
change-view-tile-open = nicht angepasste Stellen
change-view-tile-files = betroffene Dateien

change-view-col-symbol = Symbol
change-view-col-change = Änderung
change-view-col-visibility = Sichtbar
change-view-col-usages = Verw.
change-view-col-state = Zustand
change-view-col-file = Datei

change-view-kind-removed = entfernt
change-view-kind-added = neu
change-view-kind-moved = verschoben
change-view-kind-signature = Signatur
change-view-kind-visibility = Sichtbarkeit
change-view-kind-content = Inhalt

change-view-visibility-public = öffentlich
change-view-visibility-protected = geschützt
change-view-visibility-internal = intern
change-view-visibility-private = privat

change-view-state-open = { $count } offen
change-view-state-adapted = angepasst
change-view-state-none = keine Prüfliste

change-view-table-label = Symbole, die die Änderung trifft
change-view-none = Die beiden Scans unterscheiden sich in keinem Symbol.
change-view-same = Nichts geändert: beide Scans haben denselben Stand, weil zwischen ihnen keine Datei geändert wurde.

change-view-detail-signature = Geänderte Signatur
change-view-detail-visibility = Geänderte Sichtbarkeit
change-view-detail-removed = Entferntes Symbol
change-view-detail-added = Neues Symbol
change-view-detail-content = Geänderter Inhalt
change-view-detail-moved = Verschobenes Symbol
change-view-detail-narrowed = Die Sichtbarkeit wurde enger.
change-view-detail-no-checklist = Nur der Inhalt hat sich geändert oder das Symbol ist neu: Seine Verwendungen können durch diese Änderung nicht brechen, darum gibt es keine Prüfliste.
change-view-detail-label = Verwendungen des gewählten Symbols
change-view-usages-count = { $count ->
    [one] { NUMBER($count) } Verwendung
   *[other] { NUMBER($count) } Verwendungen
}
change-view-usages-certain = { $count } sicher
change-view-usages-probable = { $count } wahrscheinlich
change-view-usages-cut = { $shown } von { $total } Verwendungen sind aufgelistet.
change-view-usages-all-adapted = Keine Stelle ist mehr offen: Jede Verwendung wurde in der Änderung angepasst.
change-view-filter-open = Nur offene · { $count }
change-view-filter-all = Alle · { $count }
change-view-filter-label = Welche Verwendungen aufgelistet werden
change-view-group-open = { $count } offen
change-view-group-adapted = angepasst
change-view-status-not-touched = nicht angefasst
change-view-status-adapted = in der Änderung angepasst
change-view-confidence-certain = sicher
change-view-confidence-probable = wahrscheinlich
change-view-checklist-note = „Nicht angefasst“ ist die Prüfliste: Stellen, die das geänderte Symbol verwenden, aber in diesem Unterschied nicht geändert wurden.

change-view-violations = { $count ->
    [one] { NUMBER($count) } neuer Schichtverstoß
   *[other] { NUMBER($count) } neue Schichtverstöße
}
change-view-cycles = { $count ->
    [one] { NUMBER($count) } neuer Schichtzyklus
   *[other] { NUMBER($count) } neue Schichtzyklen
}
change-view-edge = { $from } nutzt { $to } ({ $count } Verweise)
change-view-critical = Systemkritisch
change-view-critical-line = { $name } ist als systemkritisch markiert und wurde { $outcome }.
change-view-outcome-removed = entfernt
change-view-outcome-moved = verschoben
change-view-outcome-changed = geändert
change-view-cut = Die Antwort wurde auf die eingestellte Grenze gekürzt; die Zahlen zählen die aufgelisteten Einträge.

change-view-card-added = Neu
change-view-card-added-hint = Symbole
change-view-card-changed = Geändert
change-view-card-changed-hint = Signatur { $signature } · Sichtbarkeit { $visibility } · Inhalt { $content }
change-view-card-changed-hint-cut = Von den { $shown } aufgelisteten: Signatur { $signature } · Sichtbarkeit { $visibility } · Inhalt { $content }
change-view-card-removed = Entfernt
change-view-card-removed-hint = Aufrufer prüfen
change-view-card-moved = Verschoben
change-view-card-moved-hint = in eine andere Datei
change-view-list-label = Die Symbole des Unterschieds
change-view-compare-note = Zwei Stände desselben Projekts. „Auswirkung“ zeigt die betroffenen Stellen.

change-view-filter-bar-label = Symbole filtern
change-view-filter-query-label = Nach qualifiziertem Namen suchen
change-view-filter-query-placeholder = Nach Namen suchen
change-view-filter-cause-label = Art der Änderung
change-view-filter-cause-all = Alle Änderungen
change-view-filter-cause-added = Neu
change-view-filter-cause-changed = Geändert
change-view-filter-cause-removed = Entfernt
change-view-filter-cause-moved = Verschoben
change-view-filter-kind-label = Art des Symbols
change-view-filter-kind-all = Alle Arten
change-view-filter-public = Nur öffentliche
change-view-filter-only-open = Nur mit offenen Stellen
change-view-filter-reset = Filter zurücksetzen
change-view-filter-reset-off = Es ist kein Filter gesetzt, also gibt es nichts zurückzusetzen.
change-view-filter-only-open-off = Der Vergleich hat keine Checkliste, also keine offenen Stellen.
change-view-filter-count = { $shown } von { $listed }
change-view-filter-count-cut = { $shown } von { $listed } aufgelisteten ({ $total } insgesamt)
change-view-filter-empty = Kein Symbol passt zum Filter.

# Die Ansicht „Kennzahlen“ (ADR 0114, Mock 43 a).
change-view-tab-metrics = Kennzahlen
change-metrics-reading = Die Kennzahlen werden gelesen …
change-metrics-load-failed = Die Kennzahlen konnten nicht gelesen werden.
change-metrics-card-worse = Verschlechtert
change-metrics-card-worse-hint = Objekte, { $crossing } über einer Grenze
change-metrics-card-better = Verbessert
change-metrics-card-better-hint = Objekte, { $crossing } unter die Grenze
change-metrics-card-same = Gleich geblieben
change-metrics-card-same-hint = nicht aufgelistet
change-metrics-card-same-unknown = eine Liste wurde gekürzt
change-metrics-card-new = Nicht vergleichbar
change-metrics-card-new-hint = neu { $added } · entfernt { $removed }
change-metrics-filter-label = Kennzahl
change-metrics-filter-all = Alle
change-metrics-metric-loc = Zeilen
change-metrics-metric-cc = Komplexität
change-metrics-metric-mi = Wartbarkeit
change-metrics-metric-ce = Kopplung aus
change-metrics-metric-instability = Stabilität
change-metrics-level-label = Ebene
change-metrics-level-member = Member
change-metrics-level-type = Typen
change-metrics-level-module = Module
change-metrics-list-worse = Verschlechtert
change-metrics-list-better = Verbessert
change-metrics-col-object = Objekt
change-metrics-col-values = Von → nach
change-metrics-unavailable = Diese Kennzahl gibt es nur für Typen und Module.
change-metrics-card-new-unknown = Vergleich nicht gelesen
change-metrics-list-label = Objekte, deren Kennzahlen sich bewegt haben
change-metrics-crossing-into = über der Grenze
change-metrics-crossing-out = unter der Grenze
change-metrics-none = Kein Objekt hat sich über seine Schwelle hinaus bewegt.
change-metrics-note = Verglichen werden Typen, Member und Module, die in beiden Scans vorkommen (gleicher Schlüssel wie im Vergleich). Neue und entfernte Objekte stehen in der Ansicht „Vergleich“. Gezählt wird erst ab einer Schwelle (Einstellung compare.metric_*), damit ein Zeilenwechsel nicht als Verschlechterung gilt.
