# Changelog

All notable changes of CodeSkimr. The format follows [Keep a Changelog](https://keepachangelog.com/), the versions follow [SemVer](https://semver.org/).
Each section is generated from the Conventional Commit titles of the squash-merged pull requests with `tools/release/changelog.py` (see `docs/release.md`); the titles are written in German or English, as in the pull requests.

## [0.9.0-preview.1] - 2026-11-14

### Added

- Budgets messbar machen: CPU-Zeit, Suchbenchmark, Grenze für häufige Fingerabdrücke (#141, #23, #144, #99) ([#294](https://github.com/stacknoise/CodeSkimr/pull/294))
- Theme und Sprache serverseitig speichern, Editor-Link kodiert Sonderzeichen ([#316](https://github.com/stacknoise/CodeSkimr/pull/316))
- find_constants nennt auch Konfigurations- und Übersetzungsschlüssel (#100) ([#308](https://github.com/stacknoise/CodeSkimr/pull/308))
- Zugangsdaten in der UI anlegen und beim neuen Projekt wählen (WP-76, #161) ([#314](https://github.com/stacknoise/CodeSkimr/pull/314))
- eingebettete App fertigstellen (Export, Start-Test, Windows-Rechte, Härtung) ([#292](https://github.com/stacknoise/CodeSkimr/pull/292))
- GitLab-Token-Refresh für Git-Zugangsdaten (WP-76, #161) ([#295](https://github.com/stacknoise/CodeSkimr/pull/295))
- Rust-impl-Blöcke gehören zu ihrem Typ (Container ohne Symbol, ADR 0039) ([#288](https://github.com/stacknoise/CodeSkimr/pull/288))
- parse time per language in the scan screen ([#299](https://github.com/stacknoise/CodeSkimr/pull/299))
- check_conventions vergleicht Duplikate mit dem gespeicherten Projektstand (#94) ([#287](https://github.com/stacknoise/CodeSkimr/pull/287))
- UI-Restabweichungen M3 schließen (#171, #178, #193, #158, #159, #140) ([#281](https://github.com/stacknoise/CodeSkimr/pull/281))
- Konstanten-Signatur mit voller Deklaration (Java, C#), Längengrenze für Signaturen ([#280](https://github.com/stacknoise/CodeSkimr/pull/280))
- Release-Ablauf für Vorabversionen (Windows/Linux-Pakete) ([#267](https://github.com/stacknoise/CodeSkimr/pull/267))
- Stufe-2-Messung für Java und C#, Konstruktoraufrufe als Typverwendung, Überladungsköpfe nur ohne eigenes Symbol ([#260](https://github.com/stacknoise/CodeSkimr/pull/260))
- qualified names of namespaced languages without the file ([#254](https://github.com/stacknoise/CodeSkimr/pull/254))
- Werkzeugprüfung beim Laden eines Projekts anzeigen ([#206](https://github.com/stacknoise/CodeSkimr/pull/206))
- Stufe 2 bestätigen und widerrufen per CLI, Blueprint und ADR 0033 nachgezogen (B2) ([#235](https://github.com/stacknoise/CodeSkimr/pull/235))
- Genauigkeitsmessung für Rust (rust-analyzer) und Java (javac) ([#236](https://github.com/stacknoise/CodeSkimr/pull/236))
- Ebene „Typen“ und Typ-Fokus der Abhängigkeitskarte ([#233](https://github.com/stacknoise/CodeSkimr/pull/233))
- zweiter Durchlauf im Scan, der Verweise der Stufe 2 aus SCIP-Indexen übernimmt (B1) ([#228](https://github.com/stacknoise/CodeSkimr/pull/228))
- Codezeile je Fundstelle in der Trefferliste (#157), Platz unter dem Graphen für die Legende ([#227](https://github.com/stacknoise/CodeSkimr/pull/227))
- Indexer für Stufe 2 der Verweise starten (Runner, Zustimmung, Rechte, Grenzen) ([#221](https://github.com/stacknoise/CodeSkimr/pull/221))
- Tiefenstufen der Namespace-Ebene auf der Abhängigkeitskarte ([#215](https://github.com/stacknoise/CodeSkimr/pull/215))
- SCIP-Import für Verweise der Stufe 2 (Decoder, Abbildung, Ersetzen je Datei) (`5d192f1`)
- Sprachtabelle mit Stufe und Symbolen, Branch/Commit und serverseitige Stufendauer auf dem Scan-Bildschirm ([#214](https://github.com/stacknoise/CodeSkimr/pull/214))
- Anzahl der Index-Einträge im Fuß der Schnellsuche (Teil von #178) ([#212](https://github.com/stacknoise/CodeSkimr/pull/212))
- Scan abbrechen (Knopf und Esc) und Kernanzahl auf dem Scan-Bildschirm ([#211](https://github.com/stacknoise/CodeSkimr/pull/211))
- Dokumentation und Kennzahlen in der Vorschau der Schnellsuche (Teil von #178) ([#205](https://github.com/stacknoise/CodeSkimr/pull/205))
- Ebene eines Symbols mit G und Strg+Enter in der Abhängigkeitskarte zeigen (Teil von #171, #178) ([#203](https://github.com/stacknoise/CodeSkimr/pull/203))
- Leistungsbildschirm zeigt gemessene Zeiten, Statusleiste die Abfragezeit ([#204](https://github.com/stacknoise/CodeSkimr/pull/204))
- Schichtbänder und Namespace-Ebene in der Abhängigkeitskarte ([#199](https://github.com/stacknoise/CodeSkimr/pull/199))
- Symbol mit E und Alt+Enter im Editor öffnen (Teil von #171) ([#200](https://github.com/stacknoise/CodeSkimr/pull/200))
- Zugangsdaten über den Server verwalten und mit Git-Projekten verwenden (WP-76, #161, #184) ([#196](https://github.com/stacknoise/CodeSkimr/pull/196))

### Fixed

- Lücken der C#- und TypeScript-Sprachpakete schließen (#77, #183, #81) ([#317](https://github.com/stacknoise/CodeSkimr/pull/317))
- Indexer-Härtung Stufe 2, Prozess und Dateien (#222, #223, #229) (`a6bb5c2`)
- letzte Sicherheitsbefunde für Secrets und Server (#51, #52, #68, #273, #269) ([#296](https://github.com/stacknoise/CodeSkimr/pull/296))
- Sicherheitsmeldungen schließen (minimatch, CodeQL-Warnung in secret_url) ([#305](https://github.com/stacknoise/CodeSkimr/pull/305))
- C# Zugriff über ?. als Verwendung erfassen ([#303](https://github.com/stacknoise/CodeSkimr/pull/303))
- retry the switch to the write-ahead log on a new database file ([#304](https://github.com/stacknoise/CodeSkimr/pull/304))
- Release-Lauf ersetzt einen übrig gebliebenen Entwurf ([#293](https://github.com/stacknoise/CodeSkimr/pull/293))
- scan limits for foreign projects, parallel scans and git inputs (#54, #125) ([#286](https://github.com/stacknoise/CodeSkimr/pull/286))
- Oberfläche baut unter Windows (Dateinamen Preview und preview kollidierten) ([#285](https://github.com/stacknoise/CodeSkimr/pull/285))
- C#-Auflöser erkennt mehr Empfängerformen und verteilt base-Aufrufe nicht mehr ([#283](https://github.com/stacknoise/CodeSkimr/pull/283))
- Gleichzeitige Erstöffnung der Datenbank scheitert nicht mehr an der Migration ([#284](https://github.com/stacknoise/CodeSkimr/pull/284))
- MCP rescan teilt Scan-Sperre und Id-Vergabe mit der UI (#91, #121, #155) ([#271](https://github.com/stacknoise/CodeSkimr/pull/271))
- server limits for event streams, connections and slow clients (#88) ([#268](https://github.com/stacknoise/CodeSkimr/pull/268))
- Watch-Modus ignoriert den Datenordner im beobachteten Ordner ([#279](https://github.com/stacknoise/CodeSkimr/pull/279))
- MCP-Start lädt Sprachpakete nur einmal, e2e-Baumtest wartet auf den Cursor ([#270](https://github.com/stacknoise/CodeSkimr/pull/270))
- Projekt anlegen vergibt die Id atomar, mcp legt für nie gescannte Ordner nichts an ([#277](https://github.com/stacknoise/CodeSkimr/pull/277))
- C#-Verwendungen in den Zweigen eines Bedingungsausdrucks (#263) ([#264](https://github.com/stacknoise/CodeSkimr/pull/264))
- Projektliste fällt nicht mehr aus, wenn ein gespeicherter Werkzeugbericht unlesbar ist ([#265](https://github.com/stacknoise/CodeSkimr/pull/265))
- Basic-Stufe merkt kaputtes Paket, Ruby def in if/begin, C Funktionszeiger-Rückgabe ([#266](https://github.com/stacknoise/CodeSkimr/pull/266))
- Stufe 2 bildet Pfeilfunktionen, Getter/Setter, lokale Funktionen und @overload ab (#247, #248, #249) ([#256](https://github.com/stacknoise/CodeSkimr/pull/256))
- Stufe 2 liest Self (impl#[Typ]) und Trait-Methoden aus SCIP-Indexen (#240, #241) ([#245](https://github.com/stacknoise/CodeSkimr/pull/245))
- Mindestversion von scip-typescript auf 0.3.11 anheben (#209) ([#253](https://github.com/stacknoise/CodeSkimr/pull/253))
- scip-python mit --project-version und ohne wirkungsloses Projektargument aufrufen ([#251](https://github.com/stacknoise/CodeSkimr/pull/251))
- zwei Typen gleichen Namens auf der Typ-Ebene der Karte unterscheiden ([#238](https://github.com/stacknoise/CodeSkimr/pull/238))
- Legende, feste Graphhöhe und Spalte "Ruft auf / liest" auf dem Verwendungen-Bildschirm ([#213](https://github.com/stacknoise/CodeSkimr/pull/213))
- Stufe 1 behält alle Treffer und kennzeichnet sie als ungeprüft ([#186](https://github.com/stacknoise/CodeSkimr/pull/186))
- kein doppelter Schrägstrich im Editor-Link (Teil von #171) ([#201](https://github.com/stacknoise/CodeSkimr/pull/201))

### Documentation

- fehlende Fehlercodes in Blueprint-Anhang A (#46) ([#318](https://github.com/stacknoise/CodeSkimr/pull/318))
- Neulauf der privaten Referenz im Abnahmebericht M1 und M2 (WP-28, WP-38) ([#302](https://github.com/stacknoise/CodeSkimr/pull/302))
- Handbuch auf Stand von Ebene 2 und Werkzeugprüfung bringen (WP-81) ([#278](https://github.com/stacknoise/CodeSkimr/pull/278))
- Besitzer der Fehlercode-Gruppe RES in Blueprint und Registry angleichen ([#272](https://github.com/stacknoise/CodeSkimr/pull/272))
- Handbuch um HTML-Export ergänzen (WP-81) ([#180](https://github.com/stacknoise/CodeSkimr/pull/180))
- Blueprint 16.1, 17.2 und Anhang A mit den gemessenen Stufe-2-Zahlen ([#262](https://github.com/stacknoise/CodeSkimr/pull/262))
- ADR 0036 Namen ohne Dateimodul bei Sprachen mit Namensräumen (#137) ([#239](https://github.com/stacknoise/CodeSkimr/pull/239))
- M4-Abnahme wiederholt (Budgets und Mock-Vergleich) ([#226](https://github.com/stacknoise/CodeSkimr/pull/226))

### Tests

- Genauigkeitsmessung Stufe 2 für TypeScript und Python (Spike) ([#252](https://github.com/stacknoise/CodeSkimr/pull/252))
- Genauigkeitsmessung für Stufe 2 mit von Hand erzeugtem SCIP-Index ([#242](https://github.com/stacknoise/CodeSkimr/pull/242))

### Maintenance

- Release-Vorbereitung (macOS-Pflicht-Check, Lockfile-Prüfung, macOS-Paket, Probelauf) ([#300](https://github.com/stacknoise/CodeSkimr/pull/300))
- forbid silencing no_literals and check the lint workspace in deps ([#313](https://github.com/stacknoise/CodeSkimr/pull/313))
