# HTML-Export

Der Export steht auf zwei Wegen zur Verfügung: als Knopf **HTML exportieren** in der App und als Befehl `codeskimr export`. Beide schreiben eine **eigenständige HTML-Datei** mit Explorer, Abhängigkeitskarte, Verwendungen und Schnellsuche eines gescannten Projekts. Die Datei braucht keinen Server und kein Netzwerk: Sie öffnen sie von der Platte, schicken sie per Mail oder legen sie in ein Wiki. Nutzen Sie sie für Dokumentation und für die Planung von Änderungen mit Menschen, die CodeSkimr nicht betreiben.

## In der App

Öffnen Sie das Projekt (Projekte, Zeile wählen, Enter). In der Kopfleiste steht rechts neben dem Namen und dem Zweig der Knopf **HTML exportieren**. Ein Klick erzeugt die Seite aus dem neuesten Scan, und der Browser lädt sie als `<Projektname>.html` in seinen Download-Ordner. Nichts wird an einen anderen Ort gesendet. Der Knopf ist grau, solange das Projekt nicht gescannt ist. Dasselbe geht über die Schnellsuche (`Strg K`, dann `>` und „Projekt als HTML exportieren“). Während der Export läuft, zeigt der Knopf die vergangenen Sekunden, und daneben steht **Abbrechen**. Bei großen Projekten kann der Export mehrere Minuten dauern, weil für jedes Symbol die Verwendungen gelesen werden. Nach fünf Minuten gibt die App das Warten auf und sagt es neben dem Knopf; dann hilft der Befehl `codeskimr export` in der Konsole, der kein Zeitlimit hat. Schlägt der Export fehl, steht der Grund mit Fehlercode neben dem Knopf.

## Auf der Kommandozeile

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
- Die Verwendungen jedes Symbols (Aufrufer und Aufgerufene, nur direkte Verwendungen, Tests eingeschlossen; die Filter der Werkzeugleiste wirken in der Seite; die Tiefenauswahl gibt es nicht) und die Auswirkung einer Änderung.
- Die Abhängigkeitskarte, wie CodeSkimr sie angeordnet hat.
- Name, Commit und Datum des Scans.

Die Seite sieht aus und verhält sich wie die App. Die Navigation nutzt das Adressfragment (`#/graph`) und funktioniert deshalb aus einer Datei.

## Was nicht darin steht

- **Kein Quelltext.** Die Code-Spalte sagt das. Quelltext braucht das Recht, Quelltext zu lesen, und wird nicht exportiert.
- Kein Ordner und keine Adresse des Projekts, kein Installations-Token, nichts aus dem Schlüsselspeicher.

> **Vor dem Weitergeben prüfen:** Signaturen und Dokumentationskommentare sind Text aus Ihrem Quelltext. Stehen in einem Projekt Geheimnisse oder interne Bemerkungen darin, stehen sie auch in der Seite. Sehen Sie sich einen Export an, bevor er Ihr Team verlässt.

## Ungeprüfte Verwendungen bleiben sichtbar

Jede Verwendung behält ihre Markierung (geprüft, ungeprüft, vermutet), und die Seite zeigt denselben Hinweis wie die App, wenn Ebene 2 keine der Fundstellen geprüft hat ([Verwendungen](verwendungen.md#verlässlichkeit-geprüft-ungeprüft-vermutet)). Ein Streifen unter der Kopfleiste zählt die geprüften und ungeprüften direkten Verwendungen des ganzen Exports, sodass eine Seite, die Monate später gelesen wird, noch sagt, wie weit man ihr trauen kann.

## Grenzen

- Es ist eine Momentaufnahme. Scannen, Projekte hinzufügen und Einstellungen gehören nicht dazu.
- Aufgelistet werden nur Typen und die Member von Typen, wie im Explorer; freie Funktionen außerhalb eines Typs nicht.
- Die Schnellsuche ist ein einfacher Abgleich auf Namen (Anfang, Teil, Buchstaben in Reihenfolge), nicht die Rangfolge der App.
- Die Größe wächst etwa linear mit den Verweisen, weil die direkten Verwendungen jedes Symbols eingebettet sind. Als Anhaltspunkt: etwa 1,4 MB bei 400 Symbolen, etwa 5 MB bei 1.300 und etwa 64 MB bei 7.400 (der ganze CodeSkimr-Quelltext). Sehr große Projekte ergeben Dateien von vielen Megabyte. Wege über weitere Symbole (Tiefe 2 und mehr) gibt es nur in der App: Sie wachsen viel schneller als das Projekt und machten die Datei zu groß zum Öffnen.

## Fehler

| Code | Bedeutung | Was tun |
|---|---|---|
| `PRJ-201` | das Projekt ist nicht eingetragen | zuerst eintragen und scannen |
| `MCP-203` | das Projekt wurde nicht gescannt | scannen |
| `EXP-201` | die gebaute Export-Seite fehlt oder ist unbrauchbar | Programm mit gebauter Oberfläche verwenden (`npm run build` in `web/`, dann das Programm bauen), `ui.dir` prüfen |
| `EXP-601` | die Datei kann nicht geschrieben werden | Pfad, Rechte und freien Platz prüfen |
