# Projektübersicht.

projects-title = Projekte
projects-summary =
    { $projects ->
        [one] { $projects } Projekt
       *[other] { $projects } Projekte
    } · { $types ->
        [one] { $types } Typ
       *[other] { $types } Typen
    }
projects-new = Neues Projekt
projects-new-title = Neues Projekt
projects-field-source = Quelle
projects-field-branch = Branch
projects-field-name = Anzeigename
projects-name-hint = Nur eine Beschriftung: Ordner und Nummer bleiben gleich. Leer lassen, dann gilt der vorgeschlagene Name.
projects-rename = { $name } umbenennen
projects-rename-button = Umbenennen
projects-rename-field = Neuer Anzeigename für { $name }
projects-rename-save = Speichern
projects-rename-cancel = Abbrechen
projects-add-and-scan = Hinzufügen & scannen
projects-column-project = Projekt
projects-column-source = Quelle
projects-column-languages = Sprachen
projects-column-types = Typen
projects-column-loc = LOC
projects-column-last-scan = Letzter Scan
projects-column-status = Status
projects-hint-keys = Enter öffnet · R scannt neu · N neues Projekt

projects-source-placeholder = Ordner-Pfad oder Git-URL einfügen – oder Ordner hierher ziehen
projects-source-folder = Ordner
projects-source-git = Git
projects-status-current = aktuell
projects-status-running = Scan läuft
projects-status-unscanned = nicht gescannt
projects-scan-full = vollständig
projects-scan-incremental = inkrementell
projects-scan-seconds = { $seconds } s
projects-scan-time = { $mode } · { $seconds }
projects-rescan = { $name } neu scannen
projects-adding = Scanne …
projects-drop-without-path = Der Browser verrät den Pfad eines abgelegten Ordners nicht. Füge den Pfad stattdessen in das Feld ein.
projects-load-failed = Die Projektliste konnte nicht geladen werden.
projects-open = { $name } öffnen
projects-list-label = Projekte
projects-language-mix = { $name } { $percent } %
projects-remove-button = Entfernen
projects-remove = Projekt { $name } entfernen
projects-remove-question = { $name } aus der App entfernen? Scans, Ergebnisse und Notizen des Projekts werden gelöscht. Die Dateien des Projekts bleiben unberührt.
projects-remove-confirm = Endgültig entfernen
projects-remove-cancel = Abbrechen
projects-remove-ask = Entfernung bestätigen
projects-never = nie
projects-header = { $summary } · { $hints }

projects-tools-title = Werkzeuge für genaue Verwendungen (Stufe 2): { $name }
projects-tools-missing-count =
    { $count ->
        [one] 1 Werkzeug fehlt
       *[other] { $count } Werkzeuge fehlen
    }
projects-tools-warning = Ohne diese Werkzeuge bleiben die Verwendungen der genannten Sprachen ungeprüft.
projects-tools-state-missing = fehlt
projects-tools-state-not-runnable = gefunden, startet aber nicht: { $found }
projects-tools-state-not-runnable-silent = gefunden, startet aber nicht
projects-tools-state-too-old = zu alt: { $found } installiert, { $required } gefordert
projects-tools-state-too-new = zu neu: { $found } installiert, mehr, als das Projekt erlaubt (es nennt { $required })
projects-tools-state-unknown = Version nicht lesbar (gefordert: { $required })
projects-tools-for = für { $languages }
projects-tools-purpose = Wofür: { $text }
projects-tools-install = So bekommst du es: { $text }
projects-tools-ready-languages = Stufe 2 kann laufen für: { $languages }
projects-tools-all-ready = Alle Werkzeuge sind vorhanden.
projects-tools-no-level2 = Keine Sprache dieses Projekts hat schon Stufe 2.
projects-tools-unchecked = Die Werkzeuge wurden noch nicht geprüft.
projects-tools-check-again = Erneut prüfen
projects-tools-checking = Prüfe …
projects-tools-checked = Zuletzt geprüft: { $when }

projects-credential-label = Zugangsdaten
projects-credential-none = Keine (Git-Anmeldung des Systems)
projects-credential-new = Neue Zugangsdaten speichern
projects-credential-name = Name
projects-credential-user = Benutzername
projects-credential-secret = Token oder Passwort
projects-credential-save = Zugangsdaten speichern
projects-credential-saving = Speichere …
projects-credential-saved = { $name } gespeichert
projects-credential-hint = Das Geheimnis landet im Schlüsselspeicher dieses Rechners und wird nie wieder angezeigt.

projects-level2-column = Stufe 2
projects-level2-switch-label = Stufe 2 für { $name }
projects-level2-state-on = an
projects-level2-state-off = aus
projects-level2-state-not-enabled = nicht freigeschaltet
projects-level2-state-locked = gesperrt
projects-level2-state-unknown = unbekannt
projects-level2-title = Stufe 2 für genaue Verwendungen: { $name }
projects-level2-explain-on = Stufe 2 ist an. Der nächste Scan prüft Verwendungen genau, soweit die Werkzeuge für die Sprachen des Projekts installiert sind.
projects-level2-explain-off = Stufe 2 ist aus, Scans bleiben bei Stufe 1. Zum Einschalten bestätigst du in einem Konsolenfenster, dass Build-Werkzeuge dieses Projekts laufen dürfen.
projects-level2-explain-not-enabled = Stufe 2 ist auf diesem Rechner nicht freigeschaltet. CodeSkimr ändert diese Einstellung nie selbst. Trage diese zwei Zeilen in deine Konfigurationsdatei config.toml ein, speichere sie und starte CodeSkimr neu:
projects-level2-config-file = Datei: in deiner config.toml im Datenordner
projects-level2-explain-locked = Stufe 2 ist gesperrt. Ein Team-Server führt keine Build-Werkzeuge aus.
projects-level2-explain-unknown = Der Zustand von Stufe 2 konnte nicht gelesen werden.
projects-level2-check-again = Zustand neu lesen
projects-level2-switch-on = Einschalten
projects-level2-switch-off = Ausschalten
projects-level2-opening = Das Konsolenfenster wird geöffnet …
projects-level2-waiting-title = Bestätigung im Konsolenfenster abschließen
projects-level2-waiting-text = Um { $time } wurde ein Fenster geöffnet. Lies dort die Warnung und tippe zur Bestätigung den Projektnamen ein. Diese Seite fragt regelmäßig nach und zeigt das Ergebnis hier. Falls du nicht geklickt hast, schließe das Fenster.
projects-level2-waiting-state = Warte auf die Bestätigung …
projects-level2-waiting-cancel = Warten abbrechen
projects-level2-waiting-pause = Nach dem Öffnen eines Fensters gilt eine Pause von { $seconds } Sekunden. Wenn du das Fenster schließt und zu früh erneut klickst, antwortet der Server mit einem Pausenhinweis (API-504). Warte dann kurz und versuche es noch einmal.
projects-level2-cancelled = Das Warten wurde beendet. Es wurde nichts bestätigt, Stufe 2 bleibt aus. Du kannst erneut einschalten.
projects-level2-timed-out = Nicht bestätigt: Die Bestätigung ist nicht rechtzeitig angekommen. Schließe das Konsolenfenster, falls es noch offen ist. Du kannst es erneut versuchen, indem du einschaltest.
projects-level2-copy-title = Bestätigung in einem Terminal abschließen
projects-level2-copy-text = Auf diesem Rechner kann von hier aus kein Konsolenfenster geöffnet werden. Führe diesen Befehl in einem Terminal aus und tippe auf Nachfrage den Projektnamen ein:
projects-level2-copy = Kopieren
projects-level2-copied = Kopiert
projects-level2-command-label = Befehl zum Bestätigen von Stufe 2 für { $name }
projects-level2-revoke-ask = Stufe 2 für { $name } ausschalten?
projects-level2-revoke-text = Geprüfte Ergebnisse bleiben bis zum nächsten Scan erhalten. Der nächste Scan läuft nur mit Stufe 1.
projects-level2-revoke-yes = Ja, ausschalten
projects-level2-revoke-no = Abbrechen
projects-level2-revoking = Wird ausgeschaltet …
projects-level2-revoked = Stufe 2 ist aus. Geprüfte Ergebnisse bleiben bis zum nächsten Scan erhalten. Der nächste Scan läuft nur mit Stufe 1.
projects-level2-rescan = Erneut scannen
projects-level2-retry-wait = Warte { $seconds } Sekunden und schalte dann erneut ein.
projects-credential-settings = Zugangsdaten in den Einstellungen verwalten
projects-field-source-kind = Quelle
projects-source-placeholder-folder = Ordnerpfad einfügen – oder einen Ordner hierher ziehen
projects-source-placeholder-git = Adresse eines Git-Repositorys einfügen
projects-report-origin-terminal = Terminal
projects-report-origin-browser = Browser
projects-report-origin-watch = Watch-Modus
projects-report-origin-mcp = MCP
projects-report-line = Stufe { $level } ({ $origin })
projects-report-counts = { $checked } geprüft, { $skipped } übersprungen, { $failed } fehlgeschlagen
projects-report-stale = { $count } Dateien seitdem geändert
projects-report-failed-code = Stufe 2 fehlgeschlagen ({ $code })
projects-report-title = Letzter Lauf von Stufe 2: { $time } ({ $origin })
projects-report-none = Für dieses Projekt ist noch kein Lauf von Stufe 2 gespeichert.
projects-report-checked = { $language }: geprüft, { $files } Dateien
projects-report-checked-stale = { $language }: geprüft, { $files } Dateien, { $stale } seitdem geändert
projects-report-skipped = { $language }: übersprungen, { $reason }
projects-report-failed = { $language }: fehlgeschlagen ({ $code })
projects-report-failed-note = Der letzte Lauf ist fehlgeschlagen. Für Dateien, die sich nicht geändert haben, können Ergebnisse eines früheren Laufs bleiben.
projects-report-reason-not-permitted = dieser Scan-Weg darf keine Indexer starten
projects-report-reason-not-confirmed = Stufe 2 ist für dieses Projekt nicht bestätigt
projects-report-reason-tools-unavailable = ein Werkzeug fehlt
projects-report-reason-stale = alle Dateien haben sich seit dem Lauf geändert
projects-report-reason-cancelled = abgebrochen
projects-report-reason-busy = nicht erneut gestartet, weil ein anderer Lauf von Stufe 2 aktiv war; frühere geprüfte Ergebnisse bleiben bestehen
projects-report-reason-paused = wegen der Pause nach dem letzten Lauf nicht erneut gestartet; frühere geprüfte Ergebnisse bleiben bestehen
projects-level2-running = Stufe 2 läuft ({ $language })
projects-level2-running-importing = Stufe 2 übernimmt Ergebnisse ({ $language })
projects-level2-running-unknown = Stufe 2 läuft
projects-level2-cancel = Stufe 2 abbrechen
projects-level2-cancel-note = Beim Abbrechen bleibt das Ergebnis von Stufe 1 erhalten.
projects-level2-build-note = Stufe 2 führt die Build-Werkzeuge des Projekts aus, bei einem Git-Projekt nach dem Abholen neuen Codes. Das kann Minuten dauern.
projects-level2-stage1-only = Nur Stufe 1
projects-level2-stage1-only-hint = Ein schneller Scan ohne Build-Werkzeuge.
projects-level2-full-rescan = Vollständig neu scannen
projects-level2-full-rescan-hint = Liest alle Dateien neu ein, nicht nur die Änderungen (wie „codeskimr scan“ in der Konsole).
projects-level2-scan-title = Scan
projects-level2-finished = Stufe 2 ist beendet: { $checked } geprüft, { $skipped } übersprungen, { $failed } fehlgeschlagen.
projects-level2-not-started-not-permitted = Stufe 2 wurde nicht gestartet: Dieses Programm darf hier keine Indexer starten. Stufe 1 ist gespeichert.
projects-level2-not-started-not-confirmed = Stufe 2 wurde nicht gestartet: Stufe 2 ist für dieses Projekt nicht bestätigt. Oben einschalten, dann erneut scannen. Stufe 1 ist gespeichert.
projects-level2-not-started-busy = Stufe 2 wurde nicht gestartet: Ein anderer Lauf von Stufe 2 ist aktiv. Stufe 1 ist gespeichert; scanne erneut, wenn der Lauf beendet ist.
projects-level2-not-started-paused = Stufe 2 wurde nicht gestartet: Nach einem Lauf gilt eine Pause. Stufe 1 ist gespeichert; scanne in einer Minute erneut.
projects-browse = Durchsuchen …
projects-browse-waiting-title = Ordner-Dialog geöffnet
projects-browse-waiting-text = Wähle im Fenster des Betriebssystems einen Ordner. Das Fenster kann hinter diesem Fenster liegen. Diese Seite wartet, bis du auswählst oder abbrichst.
projects-browse-give-up = Warten abbrechen
projects-browse-done = Ordner übernommen. Prüfe den Pfad und klicke auf „Hinzufügen & scannen“.
