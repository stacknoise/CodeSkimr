# HTML-Export

`codeskimr export` schreibt eine **eigenständige HTML-Datei** mit Explorer, Abhängigkeitskarte, Verwendungen und Schnellsuche eines gescannten Projekts. Die Datei braucht keinen Server und kein Netzwerk: Sie öffnen sie von der Platte, schicken sie per Mail oder legen sie in ein Wiki. Nutzen Sie sie für Dokumentation und für die Planung von Änderungen mit Menschen, die CodeSkimr nicht betreiben.

## Benutzung

```
codeskimr export <Projekt> --html <Datei>
```

- `<Projekt>` ist die ID des Projekts (wie in der Projektliste) oder der Ordner oder die Git-Adresse, aus der es gescannt wurde.
- Das Projekt muss gescannt sein; exportiert wird der **neueste Snapshot**. Exportieren Sie, solange kein Scan läuft, sonst kann die Datei zwei Snapshots mischen.
- Der Befehl gibt aus, wohin die Datei ging und wie viele Symbole sie enthält. Eine vorhandene Datei wird ersetzt.

Der Befehl braucht die gebaute Export-Seite der Oberfläche: `npm run build` in `web/` erzeugt sie (`dist/export/export.html`). Die Seite steckt in der Programmdatei; gibt es einen UI-Ordner (`ui.dir`, siehe [Installation](installation.md)), wird stattdessen die Seite aus diesem Ordner genommen. Fehlt sie an beiden Stellen, bricht der Befehl mit `EXP-201` ab.

## Was in der Datei steht

- Der Baum aus Schichten, Namensräumen und Typen.
- Zu jedem Typ und Member: Brotkrumenpfad, Metriken, Signatur, Dokumentationskommentar, Datei und Zeilenbereich.
- Die Verwendungen jedes Symbols (Aufrufer und Aufgerufene, so tief, wie `mcp.max_depth` erlaubt, Tests eingeschlossen; die Filter der Werkzeugleiste wirken in der Seite) und die Auswirkung einer Änderung.
- Die Abhängigkeitskarte, wie CodeSkimr sie angeordnet hat.
- Name, Commit und Datum des Scans.

Die Seite sieht aus und verhält sich wie die App. Die Navigation nutzt das Adressfragment (`#/graph`) und funktioniert deshalb aus einer Datei.

## Was nicht darin steht

- **Kein Quelltext.** Die Code-Spalte sagt das. Quelltext braucht das Recht, Quelltext zu lesen, und wird nicht exportiert.
- Kein Ordner und keine Adresse des Projekts, kein Installations-Token, nichts aus dem Schlüsselspeicher.

> **Vor dem Weitergeben prüfen:** Signaturen und Dokumentationskommentare sind Text aus Ihrem Quelltext. Stehen in einem Projekt Geheimnisse oder interne Bemerkungen darin, stehen sie auch in der Seite. Sehen Sie sich einen Export an, bevor er Ihr Team verlässt.

## Ungeprüfte Verwendungen bleiben sichtbar

Jede Verwendung behält ihre Markierung (geprüft, ungeprüft, vermutet), und die Seite zeigt denselben Hinweis wie die App, wenn kein Indexer etwas geprüft hat ([Verwendungen](verwendungen.md#verlässlichkeit-geprüft-ungeprüft-vermutet)). Ein Streifen unter der Kopfleiste zählt die geprüften und ungeprüften direkten Verwendungen des ganzen Exports, sodass eine Seite, die Monate später gelesen wird, noch sagt, wie weit man ihr trauen kann.

## Grenzen

- Es ist eine Momentaufnahme. Scannen, Projekte hinzufügen und Einstellungen gehören nicht dazu.
- Aufgelistet werden nur Typen und die Member von Typen, wie im Explorer; freie Funktionen außerhalb eines Typs nicht.
- Die Schnellsuche ist ein einfacher Abgleich auf Namen (Anfang, Teil, Buchstaben in Reihenfolge), nicht die Rangfolge der App.
- Die Größe wächst mit den Verweisen, weil die Verwendungen jedes Symbols eingebettet sind. Große Projekte ergeben Dateien von vielen Megabyte.

## Fehler

| Code | Bedeutung | Was tun |
|---|---|---|
| `PRJ-201` | das Projekt ist nicht eingetragen | zuerst eintragen und scannen |
| `MCP-203` | das Projekt wurde nicht gescannt | scannen |
| `EXP-201` | die gebaute Export-Seite fehlt oder ist unbrauchbar | Programm mit gebauter Oberfläche verwenden (`npm run build` in `web/`, dann das Programm bauen), `ui.dir` prüfen |
| `EXP-601` | die Datei kann nicht geschrieben werden | Pfad, Rechte und freien Platz prüfen |
