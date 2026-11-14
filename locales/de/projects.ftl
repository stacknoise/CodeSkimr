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
projects-tools-state-too-old = zu alt: { $found } installiert, { $required } gefordert
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
