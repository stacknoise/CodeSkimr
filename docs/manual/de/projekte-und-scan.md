# Projekte und Scan

Ein **Projekt** ist eine Quelle, die CodeSkimr kennt: ein Ordner auf Ihrem Rechner oder ein Git-Repository. Ein **Scan** liest die Quelle und speichert das Ergebnis als **Snapshot**. Durchsehen, Suche, Verwendungen und MCP antworten immer aus dem neuesten Snapshot. CodeSkimr behält die letzten fünf Snapshots je Projekt.

## Im Browser

Der erste Bildschirm ist die Projektliste.

- **Projekt hinzufügen:** Tippen oder fügen Sie einen absoluten Ordnerpfad oder eine Git-Adresse in das Quellfeld ein und bestätigen. Die Taste `N` springt ins Feld. Ein Ordner oder Text, den Sie auf das Fenster ziehen, landet im Feld; manche Browser verraten den Pfad eines gezogenen Ordners nicht, dann bittet der Bildschirm, ihn einzufügen.
- **Projekt öffnen:** mit den Pfeiltasten oder der Maus auswählen und `Enter` drücken. Es wird das aktive Projekt, das Explorer, Abhängigkeitskarte und Verwendungen zeigen.
- **Neu scannen:** Taste `R` oder der kleine Knopf „neu scannen“ neben dem Status. Während ein Scan läuft, steht dort „Scan läuft“.
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

- Das Argument ist ein vorhandener Ordner oder eine Git-Adresse (`https://`, `http://`, `ssh://`, `git://`, `git@host:org/repo` oder mit der Endung `.git`). Ein Ordner wird absolut gemacht. `--branch` gilt nur für Git-Quellen und ist `main`, wenn nichts angegeben ist.
- Der Befehl trägt die Quelle (einmalig) als Projekt ein und führt einen vollständigen Scan aus. Ergebnis ist eine Zusammenfassung in zwei Zeilen, danach eine Zeile je fehlendem oder zu altem Indexer-Werkzeug und, falls Sie es eingeschaltet haben, eine Zeile je Sprache für Ebene 2 („geprüft“ oder „übersprungen (Grund)“). Mit `--json` ist es eine JSON-Zeile: `{project, snapshot, files, symbols, references, seconds, tools}`.
- **`--watch`** lässt den Befehl nach dem ersten Scan weiterlaufen. Ändern sich Dateien im Ordner, werden sie einen kurzen Moment gesammelt und inkrementell neu gescannt; jeder Neuscan gibt eine Zeile aus. Nur Ordner lassen sich beobachten (`CLI-102` bei einer Git-Adresse). Beenden mit Strg+C. Ein fehlgeschlagener Neuscan gibt seinen Fehler aus, das Beobachten geht weiter.
- **Geheimnisse in einer Git-Adresse** (Benutzername und Passwort, ein Token, eine Query oder ein Fragment) werden mit `PRJ-101` abgelehnt. Den richtigen Weg zeigt [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md) (`--credential`).

Das bloße `codeskimr` startet den Server und öffnet den Browser. Weitere Befehle: `codeskimr level2 confirm|revoke` erlaubt oder verbietet Indexer für ein Projekt ([Sprach-Indexer](indexer.md)), `codeskimr credential` verwaltet Git-Anmeldungen ([Quellen und Zugangsdaten](quellen-und-zugangsdaten.md)), `codeskimr serve` startet den lokalen Server, `codeskimr open` öffnet den Browser, `codeskimr mcp` startet den MCP-Server über Standardeingabe und -ausgabe ([MCP einrichten](mcp.md)), `codeskimr export` schreibt ein Projekt als eine HTML-Datei ([HTML-Export](export.md)), und `codeskimr check` führt die Konventionsprüfungen über einen Ordner aus, ohne Scan.

Globale Optionen für alle Befehle: `--data-dir`, `--languages`, `--config` und `--set key=value` (siehe [Installation](installation.md)).

Exit-Codes: `0` Erfolg, `1` ein Befehl ist fehlgeschlagen (`error[CODE]: Text` auf der Standardfehlerausgabe) oder `check` hat Verstöße gefunden, `2` ungültige Argumente (`CLI-101`).

## Was ein Scan überspringt

Dateien, die `.gitignore` ausschließt (auch außerhalb von Git-Repositories), die Ordner in `scan.excluded_folders`, den Ordner `.git` und Dateien unbekannter Sprachen (sie zählen nicht gegen die Größengrenze). Grenzen: Ein Projekt über `scan.max_source_mb` (Vorgabe 2048 MB) scheitert mit `SRC-501`; eine einzelne Datei, deren Parsen länger als `scan.parse_timeout_ms` (Vorgabe 5000 ms) dauert, scheitert mit `PRS-501`. Signaturen (bei Konstanten die ganze Deklaration mit Wert) sind auf `scan.max_signature_chars` Zeichen begrenzt (Vorgabe 400); Längeres wird abgeschnitten und endet mit `…`. Eine `.codeskimr.toml` in einem gescannten Projekt darf diese beiden Grenzen senken, aber nicht anheben; das können nur Ihre eigene Konfigurationsdatei und Befehlszeilen-Flags. Eine Datei, die kein gültiger Text ist (`PRS-101`), wird übersprungen. Ein Scan analysiert Dateien mit einem Thread je Kern; `scan.max_workers` (Vorgabe 0, also alle Kerne) setzt eine niedrigere Grenze, etwa um einem parallelen Build Kerne zu lassen. `scan --json` nennt neben `seconds` auch `cpu_seconds` (CPU-Zeit aller Threads).

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

`namespaces` sind Präfixe qualifizierter Namen; Trenner ist `.` oder `::`. Ein Name gehört zur ersten Schicht, deren Präfix passt. Nennen Sie äußere Schichten zuerst: Die Abhängigkeitskarte zeichnet sie oben, eine Zeile je Schicht. Schichten gelten ab dem nächsten Scan. Ohne Schichten gruppiert der Explorer alles unter einer leeren Schicht, und die Karte bleibt leer.
