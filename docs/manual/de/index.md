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
| Sprach-Indexer | Verwendungen exakt machen, indem das Werkzeug der Sprache sie prüft | [Sprach-Indexer](indexer.md) |
| MCP-Server | Code-Agenten dasselbe Wissen geben | [MCP einrichten](mcp.md) |
| Quellen und Zugangsdaten | private Git-Repositories scannen, ohne Geheimnisse irgendwo abzulegen | [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md) |
| Fehlercodes | was ein Code wie `SRC-401` bedeutet und was zu tun ist | [Fehlercodes](fehler.md) |

## Unterstützte Sprachen

Es gibt zwei Stufen.

- **Volle Stufe:** C#, Java, Kotlin, Python, Rust, TypeScript, TSX und JavaScript. Typen, Member, Signaturen, Dokumentationskommentare, Sichtbarkeit, Metriken und Verweise.
- **Basis-Stufe:** C, C++, Go, PHP und Ruby. Typen, Namensräume und Funktionen mit Position und Größe, Codezeilen, Komplexität über Entscheidungspunkte und Aufrufe per Name. Keine Dokumentationskommentare und keine Sichtbarkeit.

Der Scan-Bildschirm zeigt die Stufe jeder Sprache, die in einem Projekt gefunden wurde.

## Sprache des Programms

Oberfläche und Meldungen gibt es auf Deutsch und Englisch. Die Browser-Oberfläche hat in der Fußzeile einen Umschalter `EN | DE` und in den Einstellungen ein Sprachfeld. Die Kommandozeile folgt `CODESKIMR_LANG`, dann `LC_ALL`, `LC_MESSAGES` und `LANG`: Ein Wert, der mit `de` beginnt, ergibt Deutsch, alles andere Englisch. Unter Windows, wo `LANG` meist fehlt, setzen Sie `CODESKIMR_LANG=de`.

## Aussehen der Oberfläche

Es gibt drei Themes: Hell, Petrol (Standard) und Dunkel. Wählen Sie in den Einstellungen; die Karte zeigt eine Live-Vorschau. Beim ersten Start bekommt ein dunkles Betriebssystem das Theme Dunkel.
