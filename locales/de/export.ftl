# Der HTML-Export (Blueprint 10.2): die eine Seite, die ohne Server gelesen wird.

export-banner = Export von { $project } · Scan { $snapshot } · { $date }
export-banner-commit = Commit { $commit }
export-usages-state = Verwendungen: { $verified } geprüft, { $unchecked } ungeprüft. Ungeprüfte Verwendungen wurden nur über die Namensauflösung gefunden und können unvollständig oder falsch sein.
export-limits = Statischer Export: Der Quelltext ist nicht enthalten, und der Scan lässt sich hier nicht wiederholen.
export-nav-label = Export

# Der Knopf in der Kopfleiste und sein Eintrag in der Schnellsuche.
export-button-label = HTML exportieren
export-button-title = Lädt den neuesten Scan als eine eigenständige HTML-Seite herunter. Der Quelltext ist nicht enthalten.
export-button-needs-scan = Das Projekt zuerst scannen, dann lässt es sich exportieren.
export-button-busy = Exportiere … { $seconds } s
export-button-cancel = Abbrechen
export-button-timeout = Der Export hat zu lange gedauert und wurde beendet. Große Projekte brauchen mehrere Minuten; der Befehl `codeskimr export` in der Konsole hat kein Zeitlimit.
export-button-failed = Der Export ist fehlgeschlagen.
export-command-html = Projekt als HTML exportieren
