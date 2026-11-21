# Projekte und Scan

Ein **Projekt** ist eine Quelle, die CodeSkimr kennt: ein Ordner auf Ihrem Rechner oder ein Git-Repository. Ein **Scan** liest die Quelle und speichert das Ergebnis als **Snapshot**. Durchsehen, Suche, Verwendungen und MCP antworten immer aus dem neuesten Snapshot. CodeSkimr behält die letzten fünf Snapshots je Projekt.

## Im Browser

Der erste Bildschirm ist die Projektliste.

- **Projekt hinzufügen:** Tippen oder fügen Sie einen absoluten Ordnerpfad oder eine Git-Adresse in das Quellfeld ein und bestätigen. Die Taste `N` springt ins Feld. Ein Ordner oder Text, den Sie auf das Fenster ziehen, landet im Feld; manche Browser verraten den Pfad eines gezogenen Ordners nicht, dann bittet der Bildschirm, ihn einzufügen.
- **Ordner mit einem Dialog wählen:** unter Windows und macOS öffnet der Knopf „Durchsuchen …“ neben dem Feld den Ordner-Dialog Ihres Systems. Der gewählte Pfad erscheint im Feld; mit „Hinzufügen & scannen“ legen Sie das Projekt an. Wo es keinen Dialog gibt (zum Beispiel unter Linux), tippen oder fügen Sie den Pfad ein.
- **Projekt öffnen:** mit den Pfeiltasten oder der Maus auswählen und `Enter` drücken. Es wird das aktive Projekt, das Explorer, Abhängigkeitskarte und Verwendungen zeigen.
- **Neu scannen:** Taste `R` oder der kleine Knopf „neu scannen“ neben dem Status. Während ein Scan läuft, steht dort „Scan läuft“.
- **Projekt entfernen:** der Knopf „Entfernen“ in der Zeile des Projekts (nicht, solange ein Scan des Projekts läuft). Er fragt einmal nach („Endgültig entfernen“ oder „Abbrechen“). Danach sind das Projekt, alle seine Scans und Ergebnisse, die Ebene-2-Freigabe, die Werkzeugprüfung und Ihre Notizen und Markierungen aus CodeSkimr gelöscht. **Ihre Dateien bleiben unberührt**: bei einem Ordner-Projekt wird nichts im Ordner gelöscht, bei einem Git-Projekt wird nur die Kopie gelöscht, die CodeSkimr in seinem eigenen Datenordner angelegt hat. Gespeicherte Zugangsdaten bleiben erhalten, weil andere Projekte sie nutzen können ([Quellen und Zugangsdaten](quellen-und-zugangsdaten.md)). War es das aktive Projekt, ist danach keines aktiv. Dieselbe Quelle können Sie später wieder hinzufügen; sie wird dann vollständig neu gescannt. Läuft gerade ein Scan des Projekts, wird das Entfernen mit `PRJ-302` abgelehnt: erst abbrechen oder abwarten.
- **Werkzeuge für Ebene 2:** unter der Projektliste nennt eine Warnung jedes fehlende oder zu alte Indexer-Werkzeug, wofür es gebraucht wird und wie man es installiert, mit einem Knopf „Erneut prüfen“ ([Sprach-Indexer](indexer.md)). Sie blockiert nie einen Scan.

Dieselbe Quelle wird nur einmal eingetragen. Zwei Scans desselben Projekts können nicht gleichzeitig laufen (`PRJ-302`).

Ein relativer Ordnerpfad wird abgelehnt (`PRJ-101`); der Ordner, in dem der Server zufällig läuft, entscheidet also nie, was gescannt wird.

## Bildschirm „Scan und Leistung“

Zeigt den laufenden oder zuletzt gelaufenen Scan: die Stufe, in der er steht (Quelle holen, Dateien finden, Parsen, Abhängigkeiten auflösen, Metriken, Speichern), Dateien und Zeilen pro Sekunde, genutzte Kerne, Speicher und eine Tabelle je Sprache mit Stufe, Dateien, Symbolen und Parse-Zeit. „Letzte Scans“ listet frühere auf. „Leistungsziele“ vergleicht gemessene Werte des Programms mit seinen Budgets (Programmstart, Suche je Tastendruck, Typwechsel, Verwendungsabfrage, Kartenlayout).

Der erste Scan eines Projekts ist vollständig. Spätere Scans sind **inkrementell**: Dateien werden über einen Inhalts-Hash verglichen, und nur geänderte Dateien werden neu gelesen; ein Scan nach einer kleinen Änderung ist in einem Bruchteil einer Sekunde fertig. Die Oberfläche bleibt während eines Scans bedienbar, und Sie können einen laufenden Scan mit dem Knopf „Abbrechen“ oder `Esc` **abbrechen**; er endet, ohne einen Snapshot zu schreiben.

## Auf der Kommandozeile

```
codeskimr scan <Pfad|Git-Adresse> [--branch <Name>] [--json] [--watch] [--credential <Name>]
```

- Das Argument ist ein vorhandener Ordner oder eine Git-Adresse (`https://`, `http://`, `ssh://`, `git://`, `file://` (ein lokales Repository), `git@host:org/repo` oder mit der Endung `.git`). Ein Ordner wird absolut gemacht. `--branch` gilt nur für Git-Quellen und ist `main`, wenn nichts angegeben ist.
- Der Befehl trägt die Quelle (einmalig) als Projekt ein und führt einen vollständigen Scan aus. Ergebnis ist eine Zusammenfassung in zwei Zeilen, danach eine Zeile je fehlendem oder zu altem Indexer-Werkzeug und, falls Sie es eingeschaltet haben, eine Zeile je Sprache für Ebene 2 („geprüft“ oder „übersprungen (Grund)“). Mit `--json` ist es eine JSON-Zeile: `{project, snapshot, files, symbols, references, seconds, tools}`.
- **`--watch`** lässt den Befehl nach dem ersten Scan weiterlaufen. Ändern sich Dateien im Ordner, werden sie einen kurzen Moment gesammelt und inkrementell neu gescannt; jeder Neuscan gibt eine Zeile aus. Nur Ordner lassen sich beobachten (`CLI-102` bei einer Git-Adresse). Beenden mit Strg+C. Ein fehlgeschlagener Neuscan gibt seinen Fehler aus, das Beobachten geht weiter.
- **Geheimnisse in einer Git-Adresse** (Benutzername und Passwort, ein Token, eine Query oder ein Fragment) werden mit `PRJ-101` abgelehnt. Den richtigen Weg zeigt [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md) (`--credential`).

Das bloße `codeskimr` startet den Server und öffnet den Browser. Unter Windows und macOS läuft es im Infobereich (Tray): das Konsolenfenster lässt sich schließen, und das Symbol hat ein Rechtsklick-Menü, um den Browser erneut zu öffnen, das Log zu zeigen (`codeskimr.log` im Datenordner) und das Programm zu beenden. `codeskimr --console` bleibt im Fenster; unter Linux ist es ebenso. Weitere Befehle: `codeskimr level2 confirm|revoke` erlaubt oder verbietet Indexer für ein Projekt ([Sprach-Indexer](indexer.md)), `codeskimr credential` verwaltet Git-Anmeldungen ([Quellen und Zugangsdaten](quellen-und-zugangsdaten.md)), `codeskimr annotations` und `codeskimr alerts` verwalten Notizen, Aufgaben, kritische Markierungen und deren Meldungen ([Notizen, Aufgaben und kritische Markierungen](anmerkungen.md)), `codeskimr serve` startet den lokalen Server, `codeskimr open` öffnet den Browser (`--no-browser` schreibt nur die Startdatei und gibt ihren Pfad aus), `codeskimr mcp` startet den MCP-Server über Standardeingabe und -ausgabe ([MCP einrichten](mcp.md)), `codeskimr export` schreibt ein Projekt als eine HTML-Datei ([HTML-Export](export.md)), und `codeskimr check` führt die Konventionsprüfungen über einen Ordner aus, ohne Scan.

Globale Optionen für alle Befehle: `--data-dir`, `--languages`, `--config` und `--set key=value` (siehe [Installation](installation.md)).

Exit-Codes: `0` Erfolg, `1` ein Befehl ist fehlgeschlagen (`error[CODE]: Text` auf der Standardfehlerausgabe) oder `check` hat Verstöße gefunden, `2` ungültige Argumente (`CLI-101`).

## Was ein Scan überspringt

Dateien, die `.gitignore` ausschließt (auch außerhalb von Git-Repositories), die Ordner in `scan.excluded_folders`, den Ordner `.git` und Dateien unbekannter Sprachen (sie zählen nicht gegen die Größengrenze). Grenzen: Ein Projekt über `scan.max_source_mb` (Vorgabe 2048 MB) scheitert mit `SRC-501`; eine einzelne Datei, deren Parsen länger als `scan.parse_timeout_ms` (Vorgabe 5000 ms) dauert, wird übersprungen (`PRS-501`). Signaturen (bei Konstanten die ganze Deklaration mit Wert) sind auf `scan.max_signature_chars` Zeichen begrenzt (Vorgabe 400); Längeres wird abgeschnitten und endet mit `…`. Bei sehr großen Projekten gilt außerdem `scan.max_candidates` (Vorgabe 100): Eine Stelle im Code, die nur über ihren Namen auf mehr Symbole passt (zum Beispiel `x.id` oder `.dispose()` an einem Wert mit unbekanntem Typ), wird nicht an alle verteilt, sondern gezählt und als Warnung `RES-304` gemeldet; Stellen, die Gültigkeitsbereich, Typ oder Importe eingrenzen, sind nie betroffen, und Stufe 2 liefert dort exakte Treffer. Mit einem kleineren Wert wird der Scan großer Projekte schneller und kleiner. Eine `.codeskimr.toml` in einem gescannten Projekt darf diese drei Grenzen (`scan.max_source_mb`, `scan.parse_timeout_ms`, `scan.max_candidates`) senken, aber nicht anheben; das können nur Ihre eigene Konfigurationsdatei und Befehlszeilen-Flags. Eine Datei, die kein gültiger Text ist (`PRS-101`), wird ebenfalls übersprungen, genauso eine Datei, die während des Scans verschwindet oder gesperrt ist (`PRS-102`). Der Scan läuft in allen drei Fällen weiter. Die Zusammenfassung von `codeskimr scan` und die Antwort des MCP-Werkzeugs `rescan` nennen die ausgelassenen Dateien mit Pfad und Code, höchstens `scan.max_left_out_listed` (Vorgabe 20), die übrigen als Anzahl; der Bildschirm „Scans und Leistung“ listet dieselben Dateien des neuesten Scans, jede mit ihrem Code und dem Vermerk, ob sie im Ergebnis fehlt oder so bleibt, wie der vorige Scan sie sah (die Meldung eines Codes einmal, bei seiner ersten Datei), und nennt, wie viele weitere es gab (die Liste gehört zum Scan und wird vom nächsten ersetzt); das Protokoll nennt jede (`scan_file_skipped`). Was das Ergebnis von einer ausgelassenen Datei zeigt: Bei falscher Kodierung fehlt die Datei im Ergebnis; war sie im vorigen Scan noch gültiger Text, gelten ihre Symbole als entfernt, und kritische Markierungen darin melden das. Bei Zeitüberschreitung oder nicht lesbarer Datei behält jeder Scan, auch ein Vollscan, den Stand des vorigen Scans, damit Symbole, Anmerkungen und kritische Markierungen nicht wegen einer kurzen Störung verschwinden; der nächste Scan versucht die Datei erneut. Ein Vollscan, der so eine Datei behält, wird als inkrementeller Scan gespeichert und gezählt (`mode` ist dann `incremental`). Gibt es keinen vorigen Scan oder haben sich seither die Namensregeln der Sprachpakete geändert, fehlt die Datei im Ergebnis. Bleibt keine einzige Datei übrig (zum Beispiel ein Projekt, das ganz in Windows-1252 gespeichert ist), scheitert der Scan mit dem Code der ersten ausgelassenen Datei, statt ein leeres Ergebnis zu speichern. Ein Scan analysiert Dateien mit einem Thread je Kern; `scan.max_workers` (Vorgabe 0, also alle Kerne) setzt eine niedrigere Grenze, etwa um einem parallelen Build Kerne zu lassen. `scan --json` nennt neben `seconds` auch `cpu_seconds` (CPU-Zeit aller Threads) und `pack_load_seconds` (Zeit für das Laden der Sprachpakete, die in `seconds` nicht enthalten ist).

## Schichten

CodeSkimr zeigt Ihre Architektur als **Schichten**. Sie legen sie in der Datei `.codeskimr.toml` im Wurzelordner des gescannten Ordners fest:

```toml
[[layers]]
name = "Domain"
namespaces = ["Shop.Domain"]

[[layers]]
name = "Application"
namespaces = ["Shop.Application"]

[[layers]]
name = "Infrastructure"
namespaces = ["Shop.Infrastructure", "Shop.Data"]
```

`namespaces` sind Präfixe qualifizierter Namen; Trenner ist `.` oder `::`. Ein Name gehört zur ersten Schicht, deren Präfix passt. Nennen Sie äußere Schichten zuerst: Die Abhängigkeitskarte zeichnet sie oben, eine Zeile je Schicht. Schichten gelten ab dem nächsten Scan. Eine Schicht kann außerdem angeben, von welchen Schichten sie abhängen darf: `may_use = ["Domain"]`. Eine Abhängigkeit von einer nicht genannten Schicht zählt auf der [Abhängigkeitskarte](abhaengigkeitskarte.md) als Verstoß. Ohne `may_use` gilt keine Einschränkung; eine leere Liste heißt, dass die Schicht keine andere nutzen darf; ein unbekannter Schichtname wird mit `CFG-101` abgelehnt. Ohne Schichten gruppiert der Explorer alles unter einer leeren Schicht, und die Karte bleibt leer.

### Schichten in der Oberfläche festlegen

Sie müssen die Datei nicht von Hand schreiben. Unter **Einstellungen → Schichten bearbeiten** (auch über den Hinweis im Explorer und auf der leeren Abhängigkeitskarte) zeigt CodeSkimr die Namensräume des letzten Scans als Baum. Haken Sie die Namensräume an, die zusammengehören (ein Haken gilt samt allem darunter), geben Sie der Schicht einen Namen und legen Sie sie an. Reihenfolge ändern, umbenennen und Namensräume herausnehmen geht in der Liste rechts. **Speichern und neu scannen** schreibt die `[[layers]]`-Blöcke in die `.codeskimr.toml` Ihres Projektordners und scannt das Projekt neu, danach gelten die Schichten. Nur diese Blöcke werden ersetzt; alles andere in der Datei bleibt, ebenso `may_use` bei gleichnamigen Schichten. Kommentare innerhalb der alten `[[layers]]`-Blöcke gehen verloren. CodeSkimr schlägt dabei nichts selbst vor: Schichten entstehen nur aus Ihrer Auswahl. Ein Git-Projekt hat keinen Projektordner; dort tragen Sie die Schichten im Repository ein.
