# Die Seite „Konventionen“: die Konventionsprüfung der Konsole in der Oberfläche (ADR 0116).

nav-conventions = Konventionen
conventions-title = Konventionen
conventions-subtitle = Prüft die Dateien des Projektordners gegen die Code-Regeln, so wie der Code jetzt ist. Es braucht keinen Scan, und nichts wird gespeichert.
conventions-no-project = Wähle zuerst ein Projekt, dessen Code du prüfen willst.
conventions-run = Jetzt prüfen
conventions-running = Prüfe …
conventions-run-hint = Liest jede Datei des Projektordners und prüft sie gegen die Regeln für Länge, Komplexität, Parameter, Literale und Doppelungen. Ein großes Projekt braucht etwa so lange wie die Analyse eines Scans.
conventions-git-hint = Dieses Projekt kommt von einer Git-Adresse, darum gibt es auf diesem Rechner keinen Ordner zum Prüfen.
conventions-folder = Ordner: { $folder }
conventions-checked-at = Geprüft um { $time }
conventions-before-first = Noch kein Ergebnis. „Jetzt prüfen“ liest die Dateien, wie sie jetzt sind.
conventions-failed = Die Prüfung konnte nicht gemacht werden.
conventions-copy = Als Markdown kopieren
conventions-copied = Kopiert
conventions-copy-failed = Kopieren hat nicht geklappt
conventions-copy-none = Vor der ersten Prüfung gibt es nichts zu kopieren.
conventions-figure-files = Dateien geprüft
conventions-figure-findings = Funde
conventions-figure-unchecked = Nicht analysierbar
conventions-clean = Keine Funde in { $count ->
    [one] { NUMBER($count) } Datei
   *[other] { NUMBER($count) } Dateien
}.
conventions-col-rule = Regel
conventions-col-place = Ort
conventions-col-symbol = Funktion
conventions-col-measure = Gemessen
conventions-col-measure-hint = Der Messwert und was die Regel erlaubt
conventions-open = Öffnen
conventions-open-hint = Öffnet die Datei an dieser Zeile im Editor aus den Einstellungen.
conventions-open-unavailable = Der Editor aus den Einstellungen ist nicht verfügbar, darum lässt sich die Datei von hier nicht öffnen.
conventions-measure-lines = { $value } Zeilen, erlaubt sind { $max }
conventions-measure-complexity = Komplexität { $value }, erlaubt sind { $max }
conventions-measure-parameters = { $value } Parameter, erlaubt sind { $max }
conventions-measure-literals = { $value } Literale
conventions-measure-duplicate = { $percent } % ähnlich zu { $place }
conventions-not-checked-title = Von dieser Prüfung nicht geprüft, ein sauberes Ergebnis sagt darüber also nichts:
conventions-unchecked-title = Dateien, die nicht analysiert werden konnten:
conventions-markdown-head = Konventionsprüfung von { $project }: { $findings } Funde in { $files } Dateien
conventions-markdown-none = Keine Funde.
conventions-table-label = Funde der Prüfung
conventions-more = { $hidden } weitere Funde sind nicht aufgelistet: die Liste hält höchstens { $shown } (mcp.max_limit).
