# Texte des Vergleichs zweier Scans und der Auswirkung des Unterschieds (ADR 0097).

compare-direct-only-note = Bekannt sind nur direkte Verwendungen; Reflection, Zeichenketten und andere Repositories sieht der Vergleich nicht.

compare-scans-head = Scans des Projekts: { $count } aufbewahrt (bis zu { $keep }), durchschnittliche Dauer { $average }.
compare-scans-none = Das Projekt hat noch keinen Scan.
compare-scans-line = { $snapshot }  { $created }  { $mode }  { $files } Dateien, { $changed } geändert, { $duration }  { $commit }
compare-duration = { $ms } ms
compare-unknown = -

compare-head = Vergleich von Scan { $base } ({ $base_time }) mit Scan { $target } ({ $target_time }).
compare-counts = { $added } hinzugekommen, { $removed } entfernt, { $changed } geändert, { $moved } verschoben.
compare-added = Hinzugekommen ({ $shown } von { $total }):
compare-removed = Entfernt ({ $shown } von { $total }):
compare-changed = Geändert ({ $shown } von { $total }):
compare-moved = Verschoben ({ $shown } von { $total }):
compare-symbol-line = { $kind } { $name }  { $path }
compare-changed-line = { $kind } { $name }  { $path }  ({ $reasons })
compare-moved-line = { $kind } { $name }  { $from } -> { $path }
compare-narrowed = eingeschränkt
compare-critical = Kritische Symbole, die der Unterschied berührt:
compare-critical-line = { $kind } { $name }  { $outcome }
compare-violations = Neue Schichtverstöße:
compare-cycles = Neue Schichtzyklen:
compare-edge-line = { $from } -> { $to }  ({ $count } Verweise)
compare-cut = Die Listen wurden auf die eingestellte Grenze gekürzt; --json nennt die Gesamtzahlen.

change-head = Änderung von Scan { $base } zu Scan { $target }: { $changed } Symbole geändert oder entfernt, davon { $public } öffentlich.
change-usages = Verwendungen: { $certain } sicher, { $probable } wahrscheinlich, { $open } nicht angefasst.
change-symbol = { $kind } { $name }  { $path }  ({ $cause })
change-usage-line = { $line }  ({ $status })
change-cause-removed = entfernt
change-cause-changed = geändert: { $reasons }
change-status-adapted = angepasst
change-status-not-touched = nicht angefasst
change-no-impact = Der Unterschied ändert keine Signatur oder Sichtbarkeit und entfernt kein Symbol: keine Verwendung kann dadurch brechen.

change-md-title = ## CodeSkimr: Änderung von Scan { $base } zu Scan { $target }
change-md-changed = - Geänderte Symbole: { $changed }, davon öffentlich: { $public }
change-md-usages = - Verwendungen: { $certain } sicher, { $probable } wahrscheinlich
change-md-open = - Nicht angefasst: { $open }
change-md-checklist = ### Nicht angefasst: diese Stellen prüfen
change-md-item = - [ ] `{ $path }:{ $line }` { $kind } von `{ $symbol }` ({ $cause }), in `{ $holder }`
change-md-none = Jede Verwendung, die die Änderung brechen kann, ist in derselben Änderung angepasst.
change-md-violations = ### Neue Schichtverstöße
change-md-cycles = ### Neue Schichtzyklen
change-md-edge = - `{ $from }` -> `{ $to }` ({ $count } Verweise)
change-md-critical = ### Kritische Symbole, die die Änderung berührt
change-md-critical-line = - { $kind } `{ $name }`: { $outcome }
change-md-cut = Die Antwort wurde auf die eingestellte Grenze gekürzt; die Zahlen zählen die aufgelisteten Einträge.
change-md-note = > { $note }
