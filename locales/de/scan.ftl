# Scan und Leistung.

scan-title = Scan und Leistung
scan-state-running = Scan läuft
scan-state-done = Scan abgeschlossen
scan-state-cancelled = Scan abgebrochen
scan-mode-full = voll (erster Scan)
scan-mode-incremental = inkrementell
scan-progress = { $done } von { $total } Dateien · noch ca. { $remaining } s
scan-metric-files-per-second = Dateien/s
scan-metric-lines-per-second = Zeilen/s
scan-action-cancel = Abbrechen
scan-cancel-key = Esc
scan-metric-cores = Kerne aktiv
scan-metric-memory = Speicher
scan-pipeline = Pipeline
scan-column-language = Sprache
scan-column-tier = Stufe
scan-column-files = Dateien
scan-column-symbols = Symbole
scan-column-parse-time = Parsezeit
scan-budgets-title = Leistungsziele
scan-budgets-hint = Gemessen bei jedem Build
scan-recent-title = Letzte Scans
scan-hint-unchanged = Unveränderte Dateien werden per Hash erkannt und übersprungen.
scan-hint-responsive = Scan läuft · Oberfläche bleibt bedienbar
scan-hint-incremental-results = Ergebnisse erscheinen schrittweise

nav-scan = Scans und Leistung
scan-state-failed = Scan fehlgeschlagen
scan-state-idle = Kein Scan läuft
scan-idle-hint = Starte einen Scan auf dem Projektbildschirm; sein Fortschritt erscheint hier.
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
