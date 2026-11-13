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
