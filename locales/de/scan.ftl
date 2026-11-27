# Scan und Leistung.

scan-title = Scan und Leistung
scan-state-running = Scan läuft
scan-state-done = Scan abgeschlossen
scan-state-cancelled = Scan abgebrochen
scan-mode-full = voll
scan-mode-first = voll (erster Scan)
scan-mode-incremental = inkrementell
scan-progress = { $done } von { $total } Dateien · noch ca. { $remaining } s
scan-metric-files-per-second = Dateien/s
scan-metric-lines-per-second = Zeilen/s
scan-action-cancel = Abbrechen
scan-cancel-key = Esc
scan-metric-cores = Kerne aktiv
scan-metric-memory = Speicher
scan-progress-label = Fortschritt des Scans
scan-pipeline = Pipeline
scan-column-language = Sprache
scan-column-tier = Stufe
scan-column-files = Dateien
scan-column-symbols = Symbole
scan-column-parse-time = Parsezeit
scan-budgets-title = Leistungsziele
scan-budgets-hint = Gemessen bei jedem Build; „—“ heißt: in dieser Sitzung noch nicht gemessen
scan-recent-title = Letzte Scans
scan-hint-unchanged = Unveränderte Dateien werden per Hash erkannt und übersprungen.
scan-hint-responsive = Scan läuft · Oberfläche bleibt bedienbar
scan-hint-incremental-results = Ergebnisse erscheinen schrittweise

nav-scan = Scans und Leistung
scan-state-failed = Scan fehlgeschlagen
scan-state-idle = Kein Scan läuft
scan-idle-hint = Starte einen Scan auf dem Projektbildschirm; sein Fortschritt erscheint hier.
scan-state-starting = Scan startet
scan-starting-hint = Der Scan läuft. Der Fortschritt erscheint mit der nächsten Meldung des Servers.
scan-stage-unmeasured = Läuft; der Fortschritt dieses Schritts lässt sich nicht messen
scan-no-project = Lege zuerst ein Projekt an, dann stehen seine Scans hier.
scan-files-done = { $done } von { $total } Dateien
scan-pending = wartet
scan-column-lines = Zeilen
scan-unavailable = —
scan-stage-fetch = Quelle holen
scan-stage-fetch-detail = Quelle lokal verfügbar machen
scan-stage-discover = Dateien finden
scan-stage-discover-detail = .gitignore beachten, Sprachen erkennen, Inhalte hashen
scan-stage-analyze = Parsen
scan-stage-analyze-detail = Tree-sitter, parallel
scan-stage-resolve = Abhängigkeiten auflösen
scan-stage-resolve-detail = Importe, Namensauflösung, Schichten
scan-stage-measure = Metriken
scan-stage-measure-detail = Zeilen, Komplexität, Wartbarkeit
scan-stage-persist = Speichern
scan-stage-persist-detail = SQLite, Suchindex
scan-budget-cold-start = Programmstart
scan-budget-search-keystroke = Suche (Tastendruck)
scan-budget-explorer-switch = Typ wechseln
scan-budget-usages-query = Verwendungsabfrage
scan-budget-graph-layout = Graph-Layout
scan-budget-full-scan = Scan 100k Zeilen
scan-budget-incremental = Inkrementell
scan-budget-target = Ziel < { $ms } ms
scan-budget-measured = { $ms } ms
scan-recent-line = { $name } · { $mode }
scan-recent-result = { $files ->
    [one] { NUMBER($files) } Datei
   *[other] { NUMBER($files) } Dateien
} · { $when }
scan-memory-value = { $mb } MB
scan-stage-counts = { $text } · { $done } / { $total }
scan-stage-time = { $seconds } s
scan-tier-full = voll
scan-tier-basic = Basis
scan-left-out-title = Ausgelassene Dateien
scan-left-out-hint = Diese Dateien konnten nicht analysiert werden. Der Rest des Projekts wurde gescannt.
scan-left-out-missing = fehlt im Ergebnis
scan-left-out-kept = Stand des vorigen Scans bleibt
scan-left-out-more = { $count ->
    [one] und eine weitere Datei
   *[other] und { $count } weitere Dateien
}
scan-left-out-unnamed = { $count ->
    [one] Eine Datei wurde ausgelassen. Ihr Name erscheint nach dem nächsten Scan.
   *[other] { $count } Dateien wurden ausgelassen. Ihre Namen erscheinen nach dem nächsten Scan.
}
scan-files-unchanged = Keine Datei geändert · { $checked } geprüft

scan-history-keep = { $count ->
    [one] { NUMBER($count) } wird aufbewahrt
   *[other] { NUMBER($count) } werden aufbewahrt
}
scan-history-column-time = Zeitpunkt
scan-history-column-mode = Art
scan-history-column-duration = Dauer
scan-history-column-files = Dateien
scan-history-column-changed = Geändert
scan-history-current = aktuell
scan-history-empty = Noch kein Scan.
scan-history-note = Die Dauer wird je Scan gespeichert; ein Scan ohne Änderung schreibt keinen Eintrag und lässt Zeitpunkt und Dauer des Eintrags davor stehen.
scan-history-last = Letzter Scan
scan-history-files-changed = { $count ->
    [one] { NUMBER($count) } Datei geändert seit dem vorigen
   *[other] { NUMBER($count) } Dateien geändert seit dem vorigen
}
scan-history-average = Durchschnitt
scan-history-average-hint = letzte { $count } Scans
scan-history-compare = Mit aktuellem vergleichen
scan-history-compare-title = Diesen Scan mit dem aktuellen in der Ansicht Änderung vergleichen
scan-history-failed = Die letzten Scans konnten nicht gelesen werden.
