# Kommandozeile.

cli-error-line = Fehler[{ $code }]: { $message }
cli-scan-started = Scanne { $path } …
cli-scan-finished = { $files } Dateien in { $seconds } s gescannt.
cli-scan-tool-missing = Werkzeug fehlt für { $languages }: { $tool } – { $purpose } { $install }
cli-scan-tool-not-runnable = Werkzeug gefunden, startet aber nicht, gebraucht für { $languages }: { $tool } – { $found } { $purpose } { $install }
cli-scan-tool-not-runnable-silent = Werkzeug gefunden, startet aber nicht, gebraucht für { $languages }: { $tool } (ohne Fehlermeldung) – { $purpose } { $install }
cli-scan-tool-too-old = Werkzeug zu alt für { $languages }: { $tool } { $found } ist installiert, { $required } wird gefordert – { $purpose } { $install }
cli-scan-tool-too-new = Werkzeug zu neu für { $languages }: { $tool } { $found } ist installiert, das ist mehr, als das Projekt erlaubt (es nennt { $required }) – { $purpose } { $install }
cli-scan-tool-unknown = Version des Werkzeugs nicht lesbar für { $languages }: { $tool } (gefordert: { $required }) – { $purpose } { $install }
cli-unknown-command = Unbekannter Befehl: { $command }
cli-help-hint = Mit --help werden alle Befehle angezeigt.
cli-codes-not-found = Einen Fehlercode mit der Kennung { $id } gibt es nicht.
cli-scan-details = { $symbols } Symbole, { $references } Verweise, Snapshot { $snapshot }.
cli-scan-stage = { $stage } { $seconds } s
cli-scan-stages = Schritte: { $stages }.
cli-about = CodeSkimr macht aus Code-Repositories eine navigierbare Dokumentation.
cli-cmd-scan = Einen Ordner oder eine Git-URL scannen und eine Zusammenfassung ausgeben
cli-cmd-mcp = Das MCP-Protokoll über Standardein- und -ausgabe bereitstellen
cli-cmd-serve = Den lokalen Server starten, ohne den Browser zu öffnen
cli-serve-listening = Lauscht auf { $url }. Beenden mit Strg+C.
cli-start-keep-open = Dieses Fenster offen lassen, solange du CodeSkimr benutzt.
cli-start-tray = CodeSkimr läuft jetzt im Infobereich (Tray). Der Browser öffnet sich gleich; der erste Start kann länger dauern. Über das Menü des Symbols öffnest du den Browser oder das Log und beendest das Programm. Log: { $log }
cli-arg-console = In diesem Konsolenfenster bleiben, statt im Infobereich der Taskleiste zu laufen
cli-tray-open = CodeSkimr öffnen
cli-tray-log = Log anzeigen
cli-tray-quit = CodeSkimr beenden
cli-tray-tooltip = CodeSkimr
cli-arg-source = Ordner oder Git-URL, die gescannt werden soll
cli-arg-branch = Branch einer Git-Quelle (Standard: main)
cli-arg-json = Die Zusammenfassung als JSON ausgeben
cli-arg-project = Ordner des gescannten Projekts (Standard: der aktuelle Ordner)
cli-arg-data-dir = Ordner für die Datenbank und den Git-Cache
cli-arg-config = Benutzer-Konfigurationsdatei
cli-arg-set = Eine Einstellung für diesen Lauf überschreiben, zum Beispiel metrics.cyclomatic_good_max=4
cli-arg-languages = Ordner mit den Sprachpaketen
cli-credentials-hint = Geheimnisse in einer Git-URL (Benutzerangaben, Query oder Fragment) werden nicht angenommen: Sie würden gespeichert und ausgegeben. Nutze einen Git-Credential-Helper oder einen SSH-Schlüssel.
cli-arg-watch = Nach dem Scan weiterlaufen und bei Dateiänderungen erneut scannen
cli-watch-started = Änderungen werden beobachtet. Mit Strg+C beenden.
cli-watch-updated = { $changed } geänderte Dateien in { $millis } ms erneut gescannt.
cli-cmd-check = Dateien gegen die Code-Regeln prüfen (Länge, Duplikate, kognitive Komplexität, Parameter, Literale) und bei Funden mit 1 beenden
cli-arg-paths = Dateien oder Ordner, relativ zum aktuellen Ordner (Standard: alles)
cli-check-finding = { $path }:{ $line }: { $code } { $message }
cli-check-too-complex = { $name } hat die kognitive Komplexität { $value }, erlaubt sind höchstens { $limit }.
cli-check-too-many-parameters = { $name } hat { $value } Parameter, erlaubt sind höchstens { $limit }.
cli-check-literals = { $name } enthält { $count ->
    [one] ein Literal
   *[other] { $count } Literale
}; Werte gehören in Konstanten, Konfiguration oder Textdateien.
cli-check-layer = { $name } in der Schicht { $from_layer } nutzt die Schicht { $to_layer } ({ $count ->
    [one] ein Verweis
   *[other] { $count } Verweise
}), was nicht erlaubt ist.
cli-check-too-long = { $name } hat { $lines } Codezeilen, erlaubt sind höchstens { $limit }.
cli-check-duplicate = { $name } kopiert { $original } ({ $path }:{ $line }), zu { $percent } % ähnlich.
cli-check-unchecked = { $path } übersprungen: { $code } { $message }
cli-check-summary = { $files ->
    [one] 1 Datei
   *[other] { $files } Dateien
} geprüft, { $findings ->
    [one] 1 Fund
   *[other] { $findings } Funde
}.
cli-check-not-checked = Noch nicht geprüft: { $codes }.
cli-check-unmatched = Keine Datei passt zu { $path }. Prüfe den Pfad.
cli-cmd-open = CodeSkimr im Browser öffnen (der lokale Server muss laufen)
cli-arg-no-browser = Keinen Browser starten, nur die Startdatei anlegen und ihren Pfad ausgeben
cli-arg-print-url = Mit --no-browser: Adresse mit einmaligem Code ausgeben (keine Startdatei); der Code steht dann im Terminal oder Protokoll
cli-open-url-warning = Wer diese Adresse innerhalb von { $seconds } s liest, kann sich einmal anmelden. Nicht in Protokolle oder auf geteilte Bildschirme schreiben.
cli-open-ready = Startdatei: { $path } (gültig für { $seconds } s, einmal verwendbar)
cli-cmd-export = Ein gescanntes Projekt als eine eigenständige HTML-Datei exportieren (ohne Quelltext)
cli-arg-export-project = Id des Projekts oder Ordner bzw. Git-URL, aus der es gescannt wurde
cli-arg-html = Datei, in die der HTML-Export geschrieben wird
cli-export-written = Export nach { $path } geschrieben ({ $symbols } Symbole, { $kilobytes } KiB). Der Quelltext ist nicht enthalten.
cli-cmd-credential = Zugangsdaten für private Git-Repositories speichern, ersetzen oder entfernen
cli-cmd-credential-set = Zugangsdaten speichern; das Geheimnis wird vom Terminal (verdeckt) oder von der Standardeingabe gelesen
cli-cmd-credential-delete = Gespeicherte Zugangsdaten entfernen
cli-cmd-credential-login = Im Browser bei GitHub oder GitLab anmelden (OAuth Device Flow) und das Token speichern
cli-arg-id = Name der Zugangsdaten, zum Beispiel work-github
cli-arg-user = Benutzername, der zum Geheimnis gehört
cli-arg-provider = Die Plattform, bei der angemeldet wird
cli-arg-client-id = Kennung der OAuth-Anwendung, die bei der Plattform registriert ist
cli-arg-host = Adresse einer selbst betriebenen Installation, zum Beispiel https://git.example.com
cli-arg-scan-name = Anzeigename des Projekts; benennt auch ein schon eingetragenes Projekt um
cli-arg-scan-credential = Name gespeicherter Zugangsdaten, mit denen die Git-Quelle angemeldet wird (siehe Befehl credential)
cli-credential-stored = Zugangsdaten { $id } gespeichert.
cli-credential-deleted = Zugangsdaten { $id } entfernt.
cli-secret-prompt = Geheimnis (wird nicht angezeigt):
cli-credential-device-prompt = Öffne { $uri } und gib den Code { $user_code } ein. Warte auf deine Bestätigung …
cli-credential-unusable-hint = Zugangsdaten gelten nur für eine Git-Quelle, nicht für einen Ordner, und ihr Name darf nicht leer sein.
cli-scan-level2-checked = Stufe 2: { $language } geprüft ({ $files } Dateien)
cli-scan-level2-partial = Stufe 2: { $language } geprüft ({ $files } Dateien, { $stale } seit dem Scan geändert und bei Stufe 1 geblieben)
cli-scan-level2-skipped = Stufe 2: { $language } übersprungen ({ $reason })
cli-scan-level2-outside-scan = Stufe 2: { $language } Verwendungen von Code in Ordnern, die der Scan überspringt, nicht gezählt: { $count }
cli-level2-reason-not-permitted = dieser Aufrufer darf keine Indexer starten
cli-level2-reason-not-confirmed = für dieses Projekt nicht aktiviert und bestätigt
cli-level2-reason-unavailable = Indexer oder Werkzeugkette fehlt oder ist zu alt
cli-level2-reason-no-project-file = im Projektordner fehlt die Projektdatei, die der Indexer braucht (z. B. Cargo.toml oder .sln)
cli-level2-reason-stale = alle Dateien seit dem Scan geändert
cli-level2-reason-cancelled = Scan abgebrochen
cli-level2-reason-failed = fehlgeschlagen, { $code }: { $message }
cli-cmd-level2 = Stufe 2 (Indexer) für ein Projekt erlauben oder verbieten
cli-cmd-level2-confirm = Indexer für ein Projekt erlauben (fragt vorher nach)
cli-cmd-level2-revoke = Indexer für ein Projekt verbieten
cli-arg-level2-project = Id des Projekts oder Ordner oder Git-Adresse, aus der es gescannt wurde
cli-arg-level2-from-ui = Wird von der Seite im Browser gesetzt: das Fenster wurde für sie geöffnet
cli-level2-from-ui-notice = Dieses Fenster wurde von der CodeSkimr-Seite in deinem Browser geöffnet. Falls du nicht darauf geklickt hast, schließe es.
cli-level2-from-ui-wait = Drücke Enter, um dieses Fenster zu schließen.
cli-level2-warning = Stufe 2 startet die Build-Werkzeuge dieses Projekts (Compiler, Paketverwaltung und den Indexer der Sprache) auf diesem Rechner. Sie führen den Code und die Skripte des Projekts mit deinen Rechten aus: Sie können Dateien deines Benutzerkontos lesen und ändern und das Netzwerk nutzen. Die Indexer laufen nicht in einer Sandbox. Die Bestätigung gilt für Quelle, Ort und Branch des Projekts, nicht für einen Commit: Was der Branch später enthält, wird unter ihr ausgeführt. Bestätige nur für Code, dem du vertraust.
cli-level2-target = Projekt: { $project }
    Ort: { $place }
cli-level2-target-branch = Branch: { $branch }
cli-level2-prompt = Gib den Namen des Projekts („{ $project }“) ein, um Stufe 2 dafür zu erlauben, alles andere bricht ab:
cli-level2-confirmed = Stufe 2 ist für Projekt { $project } erlaubt. Sie läuft beim nächsten Scan auf der Kommandozeile.
cli-level2-declined = Nichts geändert: Stufe 2 bleibt für Projekt { $project } aus.
cli-level2-revoked = Stufe 2 ist für Projekt { $project } wieder verboten.
cli-scan-warning-ambiguous-reference = Warnung RES-301: { $count } Stellen passen auf mehrere Symbole und sind als wahrscheinlich gespeichert.
cli-scan-warning-other = Warnung { $code }: { $count }-mal.
cli-scan-warning-files-left-out = { $count ->
    [one] Warnung { $code }: Eine Datei konnte nicht analysiert werden und wurde ausgelassen.
   *[other] Warnung { $code }: { $count } Dateien konnten nicht analysiert werden und wurden ausgelassen.
}
cli-scan-left-out-file = {"  "}{ $path } ({ $code }): fehlt im Ergebnis
cli-scan-left-out-file-kept = {"  "}{ $path } ({ $code }): Stand des vorigen Scans bleibt
cli-scan-left-out-more = { $count ->
    [one] {"  "}und eine weitere Datei
   *[other] {"  "}und { $count } weitere Dateien
}
cli-cmd-annotations = Notizen, Aufgaben und Kritisch-Markierungen an Symbolen
cli-cmd-annotations-list = Die Anmerkungen eines Projekts auflisten
cli-cmd-annotations-add = Einem Symbol eine Kritisch-Markierung, eine Notiz oder eine Aufgabe hinzufügen
cli-cmd-annotations-update = Eine Anmerkung ändern oder einem anderen Symbol zuordnen
cli-cmd-annotations-remove = Eine Anmerkung samt Alarmen löschen
cli-cmd-alerts = Alarme für kritische Symbole, die sich geändert haben, entfernt wurden oder ihren Platz verloren haben
cli-cmd-alerts-list = Die Alarme eines Projekts auflisten
cli-cmd-alerts-dismiss = Einen Alarm entfernen
cli-cmd-alerts-dismiss-all = Alle offenen Alarme eines Filters entfernen; braucht --yes
cli-arg-note-project = Nummer des Projekts oder der Ordner oder die Git-Adresse, aus der es gescannt wurde
cli-arg-annotation = Nummer der Anmerkung
cli-arg-alert = Nummer des Alarms
cli-arg-symbol = Vollständiger Name des Symbols
cli-arg-symbol-kind = Art des Symbols, zum Beispiel class, method oder namespace
cli-arg-param = Parametertyp des Symbols, einmal je Parameter, um Überladungen zu unterscheiden
cli-arg-kind = Art der Anmerkung: critical, note oder task
cli-arg-status = Status einer Aufgabe: open, in_progress oder done
cli-arg-title = Titel einer Aufgabe
cli-arg-text = Text der Anmerkung oder Begründung einer Kritisch-Markierung
cli-arg-due = Fälligkeitsdatum einer Aufgabe in Sekunden seit dem 1. Januar 1970
cli-arg-assignee = Wem eine Aufgabe zugeordnet ist
cli-arg-clear-text = Den Text entfernen
cli-arg-clear-due = Das Fälligkeitsdatum entfernen
cli-arg-clear-assignee = Den Bearbeiter entfernen
cli-arg-limit = Anzahl der Zeilen; ohne Angabe die eingestellte Seitengröße
cli-arg-offset = Zeilen, die übersprungen werden, um die nächste Seite zu lesen
cli-arg-alert-state = Nur Alarme in diesem Zustand: open (Standard) oder dismissed
cli-arg-alert-kinds = Nur Alarme dieser Art: changed, removed oder orphaned; darf mehrfach stehen
cli-arg-yes = Bestätigt, dass alle passenden Alarme entfernt werden
cli-annotation-line = Nr. { $id } [{ $kind }] { $name }{ $parts ->
    [0] { "" }
   *[other] : { $summary }
}
cli-annotation-saved = Anmerkung Nr. { $id } an { $name } gespeichert.
cli-annotation-removed = Anmerkung Nr. { $id } gelöscht.
cli-annotation-none = Keine Anmerkungen.
cli-annotations-page = { $shown } von { $total } gezeigt; weiter mit --offset { $next }.
cli-alert-line = Nr. { $id } [{ $state }/{ $kind }] { $name }{ $parts ->
    [0] { "" }
   *[other] : { $detail }
}
cli-alert-none = Keine Alarme.
cli-alerts-page = { $shown } von { $total } Alarmen gezeigt, { $open } offen; weiter mit --offset { $next }.
cli-alerts-dismissed = { $dismissed ->
    [one] Ein Alarm entfernt, { $open } noch offen.
   *[other] { $dismissed } Alarme entfernt, { $open } noch offen.
}
cli-cmd-codes = Die Fehlercodes auflisten oder einen erklären
cli-arg-code = Der zu erklärende Code, zum Beispiel SRC-201 (ohne Angabe: alle Codes auflisten)
cli-codes-line = { $code }  { $message }
cli-codes-explain =
    { $code }: { $message }
    { $description }
    Gruppe { $group }, { $severity ->
        [warning] Warnung
        [info] Hinweis
       *[error] Fehler
    }, { $retryable ->
        [yes] ein neuer Versuch kann gelingen
       *[no] ein neuer Versuch hilft nicht
    }, seit Version { $since }.
cli-codes-retired = Dieser Code ist stillgelegt: nichts löst ihn mehr aus, die Nummer bleibt reserviert.

cli-cmd-scope = Zeigen oder ändern, welche Sprachen und Ordner ein Projekt erfasst
cli-arg-scope-project = Nummer des Projekts, oder der Ordner oder die Git-Adresse, von der es gescannt wurde
cli-arg-without-language = Diese Sprache aus den Scans auslassen (mehrfach möglich)
cli-arg-with-language = Diese Sprache wieder erfassen (mehrfach möglich)
cli-arg-without-folder = Diesen Ordner aus den Scans auslassen, relativ zum Projektordner (mehrfach möglich)
cli-arg-with-folder = Diesen Ordner wieder erfassen (mehrfach möglich)
cli-arg-scope-reset = Zuerst alles wieder erfassen, danach die anderen Änderungen anwenden
cli-scope-heading = Umfang von Projekt { $project }:
cli-scope-language = { $state ->
    [covered] Erfasst
   *[left_out] Ausgelassen
}: Sprache { $language } ({ $files } Dateien)
cli-scope-folder = { $state ->
    [covered] Erfasst
   *[left_out] Ausgelassen
}: Ordner { $path } ({ $files } Dateien)
cli-scope-more = { $count } weitere Ordner sind nicht aufgeführt. Mit --json kommt die ganze Antwort.
cli-scope-saved = Der Umfang ist gespeichert. Er gilt ab dem nächsten Scan; der Code bleibt ein Projekt.
cli-scope-unchanged = Der Umfang bleibt, wie er war.
# Die Abfragebefehle symbols, usages, impact und context.
cli-cmd-symbols = Symbole im neuesten Scan eines Projekts nach Namen suchen (die Antwort des MCP-Werkzeugs search_symbols)
cli-cmd-usages = Auflisten, wo ein Symbol verwendet wird oder was es verwendet (die Antwort von find_usages)
cli-cmd-impact = Auflisten, was eine Änderung eines Symbols erreichen kann (die Antwort von impact_of_change)
cli-cmd-context = Signatur, Aufrufer, Abhängigkeiten, betroffene Dateien und Anmerkungen eines Symbols in einer Antwort (die Antwort von get_context)
cli-arg-query-text = Teil des Symbolnamens; die Zeichen müssen in dieser Reihenfolge vorkommen
cli-arg-query-symbol = Id oder qualifizierter Name des Symbols
cli-arg-query-project = Id des Projekts, oder der Ordner oder die Git-URL, aus der es gescannt wurde (Standard: der aktuelle Ordner)
cli-arg-kind-filter = Nur Symbole dieser Art, etwa class, function oder constant
cli-arg-direction = incoming (wer das Symbol verwendet, Standard) oder outgoing (was es verwendet)
cli-arg-depth = Schritte, die die Suche geht (mindestens 1; ohne Angabe der eingestellte Standard)
cli-arg-include-tests = Verweise aus Testcode mitzählen
cli-arg-budget = Wie viel die Antwort enthält: short, medium (Standard) oder long
cli-query-none = Nichts gefunden.
cli-query-symbol-line = { $kind } { $name }  { $path }  (Id { $id })
cli-query-usage-line = { $path }:{ $line }:{ $column }  { $kind }, { $confidence }, in { $name }
cli-query-impact-line = Schritt { $depth }  { $kind } { $name }  { $path }
cli-query-cut = Die Antwort wurde auf die eingestellte Grenze gekürzt; mit --limit mehr, mit --json die Gesamtzahlen.
cli-query-preliminary = Vorläufig: Einige Treffer stammen nur aus der Namensauflösung und wurden von keinem Indexer (Stufe 2) geprüft.
cli-query-stale = Dateien haben sich seit dem letzten Scan geändert, die Antwort kann veraltet sein. Zuerst codeskimr scan ausführen.
cli-context-head = { $kind } { $name }  ({ $path }:{ $line })
cli-context-signature = Signatur: { $text }
cli-context-layer = Schicht: { $text }
cli-context-callers = Aufrufer ({ $shown } von { $total }):
cli-context-dependencies = Abhängig von ({ $shown } von { $total }):
cli-context-dependency-line = { $count } x { $kind } { $name }  { $path }
cli-context-files = Dateien, die eine Änderung erreichen kann ({ $shown } von { $total }):
cli-context-file = { $path }
cli-context-notes = Anmerkungen ({ $shown } von { $total }):
cli-context-note-line = { $kind }: { $text }
