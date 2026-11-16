# Schichten festlegen: Auswahl aus den erkannten Namensräumen (ADR 0065).

layers-title = Schichten
layers-subtitle = Stelle aus den erkannten Namensräumen Schichten zusammen. Die Auswahl wird in .codeskimr.toml im Projektordner gespeichert.
layers-no-project = Wähle zuerst ein Projekt, dessen Schichten du festlegen willst.
layers-not-scanned = Das Projekt wurde noch nicht gescannt, es gibt noch keine Namensräume zur Auswahl. Scanne es zuerst.
layers-not-editable = Ein Git-Projekt hat keinen Projektordner, in den die Schichten geschrieben werden könnten. Trage sie in der .codeskimr.toml des Repositorys ein.
layers-load-failed = Die Schichten des Projekts konnten nicht gelesen werden.
layers-namespaces-title = Erkannte Namensräume
layers-namespaces-note = Ein Haken wählt den Namensraum samt allem darunter aus.
layers-namespaces-empty = Der letzte Scan hat keine Namensräume gefunden.
layers-types = { $count ->
    [one] { NUMBER($count) } Typ
   *[other] { NUMBER($count) } Typen
}
layers-owned-by = in „{ $layer }“
layers-expand = { $name } aufklappen
layers-collapse = { $name } zuklappen
layers-select = { $name } auswählen
layers-selected-count = { $count ->
    [one] { NUMBER($count) } ausgewählt
   *[other] { NUMBER($count) } ausgewählt
}
layers-layers-title = Deine Schichten
layers-order-note = Die Reihenfolge zählt: Passt ein Name zu mehreren Schichten, gewinnt die erste. Äußere Schichten stehen oben.
layers-empty = Noch keine Schichten. Wähle links Namensräume aus und lege daraus eine Schicht an.
layers-new-name = Name der neuen Schicht
layers-new-action = Schicht aus Auswahl anlegen
layers-add-selection = Auswahl hinzufügen
layers-name-label = Name der Schicht
layers-move-up = { $name } nach oben
layers-move-down = { $name } nach unten
layers-remove = { $name } entfernen
layers-remove-namespace = { $name } aus der Schicht nehmen
layers-save = Speichern und neu scannen
layers-saving = Speichere und scanne …
layers-reset = Änderungen verwerfen
layers-invalid = Jede Schicht braucht einen eigenen Namen und mindestens einen Namensraum.
layers-saved = Gespeichert. Der neue Scan ist fertig.
layers-file-note = Beim Speichern werden nur die [[layers]]-Blöcke der Datei ersetzt; die Erlaubnisse „may_use“ bleiben für gleichnamige Schichten erhalten. Kommentare innerhalb der alten Blöcke gehen verloren.
layers-hint-title = Noch keine Schichten festgelegt
layers-hint-text = Ohne Schichten bleibt alles ungruppiert und die Abhängigkeitskarte ist leer.
layers-hint-action = Schichten festlegen
settings-layers-title = Schichten
settings-layers-note = Lege fest, welche Namensräume zu welcher Schicht gehören. Explorer und Abhängigkeitskarte gruppieren danach.
settings-layers-action = Schichten bearbeiten
