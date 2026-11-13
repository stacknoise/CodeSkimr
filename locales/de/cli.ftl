# Kommandozeile.

cli-error-line = Fehler[{ $code }]: { $message }
cli-scan-started = Scanne { $path } …
cli-scan-finished = { $files } Dateien in { $seconds } s gescannt.
cli-unknown-command = Unbekannter Befehl: { $command }
cli-help-hint = Mit --help werden alle Befehle angezeigt.
cli-codes-not-found = Einen Fehlercode mit der Kennung { $id } gibt es nicht.
cli-scan-details = { $symbols } Symbole, { $references } Verweise, Snapshot { $snapshot }.
cli-about = CodeSkimr macht aus Code-Repositories eine navigierbare Dokumentation.
cli-cmd-scan = Einen Ordner oder eine Git-URL scannen und eine Zusammenfassung ausgeben
cli-cmd-mcp = Das MCP-Protokoll über Standardein- und -ausgabe bereitstellen
cli-cmd-serve = Den lokalen Server starten, ohne den Browser zu öffnen
cli-serve-listening = Lauscht auf { $url }. Beenden mit Strg+C.
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
cli-cmd-check = Dateien gegen die Code-Regeln prüfen (Länge, Duplikate) und bei Funden mit 1 beenden
cli-arg-paths = Dateien oder Ordner, relativ zum aktuellen Ordner (Standard: alles)
cli-check-finding = { $path }:{ $line }: { $code } { $message }
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
cli-open-ready = Startdatei: { $path } (gültig für { $seconds } s, einmal verwendbar)
cli-cmd-credential = Zugangsdaten für private Git-Repositories speichern, ersetzen oder entfernen
cli-cmd-credential-set = Zugangsdaten speichern; das Geheimnis wird vom Terminal (verdeckt) oder von der Standardeingabe gelesen
cli-cmd-credential-delete = Gespeicherte Zugangsdaten entfernen
cli-cmd-credential-login = Im Browser bei GitHub oder GitLab anmelden (OAuth Device Flow) und das Token speichern
cli-arg-id = Name der Zugangsdaten, zum Beispiel work-github
cli-arg-user = Benutzername, der zum Geheimnis gehört
cli-arg-provider = Die Plattform, bei der angemeldet wird
cli-arg-client-id = Kennung der OAuth-Anwendung, die bei der Plattform registriert ist
cli-arg-host = Adresse einer selbst betriebenen Installation, zum Beispiel https://git.example.com
cli-arg-scan-credential = Name gespeicherter Zugangsdaten, mit denen die Git-Quelle angemeldet wird (siehe Befehl credential)
cli-credential-stored = Zugangsdaten { $id } gespeichert.
cli-credential-deleted = Zugangsdaten { $id } entfernt.
cli-secret-prompt = Geheimnis (wird nicht angezeigt):
cli-credential-device-prompt = Öffne { $uri } und gib den Code { $user_code } ein. Warte auf deine Bestätigung …
cli-credential-unusable-hint = Zugangsdaten gelten nur für eine Git-Quelle, nicht für einen Ordner, und ihr Name darf nicht leer sein.
