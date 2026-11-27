# CodeSkimr: Übersicht

CodeSkimr liest ein Code-Repository einmal ein und baut daraus eine Datenbank: welche Namensräume, Typen und Member es gibt, wer sie verwendet, wie groß und komplex sie sind und wie die Teile des Programms voneinander abhängen. Diese Datenbank durchsuchen Sie im Browser, fragen sie auf der Kommandozeile ab oder lassen sie einen Code-Agenten über MCP fragen.

Alles läuft auf Ihrem Rechner. Der Server hört nur auf `127.0.0.1`; Ihr Code wird nirgendwohin hochgeladen.

## Die Bausteine

| Baustein | Wofür | Kapitel |
|---|---|---|
| Projekte und Scan | einen Ordner oder ein Git-Repository eintragen und einlesen | [Projekte und Scan](projekte-und-scan.md) |
| Explorer und Suche | Schichten, Namensräume, Typen und Member durchsehen; Symbole schnell finden | [Explorer](explorer.md) |
| Abhängigkeitskarte | die Schichten des Programms sehen und welche welche nutzt | [Abhängigkeitskarte](abhaengigkeitskarte.md) |
| Verwendungen | wer ein Member nutzt und was bricht, wenn man es ändert | [Verwendungen](verwendungen.md) |
| Statistik | Ranglisten: die meistverwendeten Symbole, die ohne Verwendung, die komplexesten | [Statistik](statistik.md) |
| Regeln | Architekturregeln in der Projektdatei (Schichten, Verbote, Zyklen, öffentliche Schnittstelle), ihre Verstöße und das Anlegen im UI | [Regeln](regeln.md) |
| Änderung | zwei Scans vergleichen und sehen, welche Verwendungen ein Unterschied trifft, mit einer Prüfliste der nicht angefassten Stellen | [Änderung](aenderung.md) |
| Notizen, Aufgaben, kritische Markierungen | Symbole als kritisch markieren, Notizen schreiben, Aufgaben verfolgen, Meldungen erhalten, wenn sich ein kritisches Symbol ändert | [Notizen, Aufgaben und kritische Markierungen](anmerkungen.md) |
| Sprach-Indexer (Ebene 2) | Verwendungen exakt machen, indem das Werkzeug der Sprache sie prüft; standardmäßig aus | [Sprach-Indexer](indexer.md) |
| MCP-Server | Code-Agenten dasselbe Wissen geben | [MCP einrichten](mcp.md) |
| PR-Kommentar | die Auswirkung einer Änderung in jedem Pull Request | [PR-Kommentar](pr-comment.md) |
| Quellen und Zugangsdaten | private Git-Repositories scannen, ohne Geheimnisse irgendwo abzulegen | [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md) |
| HTML-Export | eine eigenständige Datei eines Projekts für Menschen ohne CodeSkimr | [HTML-Export](export.md) |
| Fehlercodes | was ein Code wie `SRC-401` bedeutet und was zu tun ist | [Fehlercodes](fehler.md) |

## Unterstützte Sprachen

Es gibt zwei Stufen.

- **Volle Stufe:** C#, Java, Kotlin, Python, Rust, TypeScript, TSX und JavaScript. Typen, Member, Signaturen, Dokumentationskommentare, Sichtbarkeit, Metriken und Verweise.
- **Basis-Stufe:** C, C++, Go, PHP und Ruby. Typen, Namensräume und Funktionen mit Position und Größe, Codezeilen, Komplexität über Entscheidungspunkte und Aufrufe per Name. Keine Dokumentationskommentare und keine Sichtbarkeit.

Der Reiter „Scan“ eines Projekts zeigt die Stufe jeder Sprache, die in einem Projekt gefunden wurde.

## Sprache des Programms

Oberfläche und Meldungen gibt es auf Deutsch und Englisch. Die Browser-Oberfläche hat in der Fußzeile einen Umschalter `EN | DE` und in den Einstellungen ein Sprachfeld. Die Kommandozeile folgt `CODESKIMR_LANG`, dann `LC_ALL`, `LC_MESSAGES` und `LANG`: Ein Wert, der mit `de` beginnt, ergibt Deutsch, alles andere Englisch. Unter Windows, wo `LANG` meist fehlt, setzen Sie `CODESKIMR_LANG=de`.

## Aussehen der Oberfläche

Die Einstellungen haben sechs Reiter: Darstellung, Analyse, Zugänge, Wirksame Einstellungen, Info und Diagnose. Das Suchfeld rechts neben den Reitern sucht in den wirksamen Einstellungen.

Es gibt drei Themes: Hell, Petrol (Standard) und Dunkel. Wählen Sie in den Einstellungen im Reiter **Darstellung**; die Karte zeigt eine Live-Vorschau. Beim ersten Start bekommt ein dunkles Betriebssystem das Theme Dunkel.

Theme und Sprache merkt sich **der Server** (Datei `preferences.json` im Datenordner, siehe [Installation](installation.md#wo-die-daten-liegen)), jeder Browser, der dieselbe Installation öffnet, zeigt also dieselbe Wahl. Der Browser behält eine Kopie, damit der erste Bildschirm ohne Anfrage auskommt; hat der Server eine Wahl gespeichert, gilt sie vor der Kopie. Die Datei enthält kein Geheimnis.

## Das Protokoll (Reiter „Diagnose“)

Wenn etwas schiefgeht, zeigt das Protokoll, was das Programm getan hat. Dafür brauchen Sie keine Konsole: Öffnen Sie die Einstellungen, Reiter **Diagnose**.

- **Stufe.** Aus, Warnungen (Standard), Info, Debug oder Trace. Die Wahl gilt sofort, ohne Neustart, und auch für ein Konsolenfenster desselben Laufs. Ab Debug werden große Scans langsamer und die Datei wird groß. Eine andere Stufe als der Standard **fällt nach 24 Stunden von selbst auf den Standard zurück**, damit ein vergessenes Trace die Platte nicht füllt; der Reiter zeigt die verbleibende Zeit. Nach einem Neustart gilt wieder der Standard (oder `CODESKIMR_LOG`, wenn Sie es gesetzt haben).
- **Datei.** Das Protokoll wird zusätzlich in `logs/codeskimr.log` im Datenordner geschrieben (siehe [Installation](installation.md#wo-die-daten-liegen)). Bei 10 MB wird die Datei umbenannt und eine neue beginnt; es bleiben höchstens fünf Dateien, also höchstens 50 MB. Zugangsdaten, Tokens und Quelltext stehen nicht darin: Sie werden vor dem Schreiben entfernt.
- **Knöpfe.** „Protokoll kopieren“ kopiert die letzten 2000 Zeilen (für eine Fehlermeldung), „Ordner öffnen“ öffnet den Ordner mit dem Programm Ihres Systems (nur auf dem Rechner der App), „Datei leeren“ leert die Datei und entfernt die alten. Darunter sehen Sie die letzten Zeilen.

Die Stufe entspricht der Umgebungsvariable `CODESKIMR_LOG` während eine im Reiter gewählte Stufe nur für das Programm selbst gilt (Bibliotheken bleiben bei Warnungen); der Reiter ersetzt dafür den Start aus einem Terminal.
