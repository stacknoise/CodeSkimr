# Die Liste der Fehlercodes zum Nachschlagen im UI; das Gegenstück zu `codeskimr codes`.

codes-title = Fehlercodes
codes-subtitle = { $count ->
    [one] { $count } Code, wie bei `codeskimr codes`
   *[other] { $count } Codes, wie bei `codeskimr codes`
}
codes-search-label = Fehlercodes durchsuchen
codes-search-placeholder = Nach Code oder Text suchen, zum Beispiel SRC-201
codes-severity-all = Alle Schweregrade
codes-severity-error = Fehler
codes-severity-warning = Warnung
codes-severity-info = Hinweis
codes-group-label = Gruppe
codes-group-all = Alle Gruppen
codes-col-code = Code
codes-col-message = Meldung
codes-col-group = Gruppe
codes-col-severity = Schwere
codes-col-retry = Neuer Versuch
codes-col-since = Seit
codes-retry-yes = Kann helfen
codes-retry-no = Nein
codes-description = Beschreibung für Entwickler (englisch): { $description }
codes-facts = Gruppe { $group }, { $severity }, neuer Versuch: { $retry }, seit Version { $since }.
codes-retired = Dieser Code ist stillgelegt: nichts löst ihn mehr aus, die Nummer bleibt reserviert.
codes-empty = Kein Code passt.
codes-note-retired = Stillgelegte Codes (ADR 0078) sind ausgeblendet. Gib die volle Nummer eines Codes ein, um ihn zu sehen.
codes-back = Zurück zu den Einstellungen
settings-codes-title = Fehlercodes
settings-codes-note = Schlage jeden Fehlercode mit seiner Erklärung nach, so wie `codeskimr codes` sie auflistet.
settings-codes-action = Fehlercodes anzeigen
