# Fehlercodes

Jeder Fehler, den CodeSkimr meldet, hat einen Code der Form `PRÄFIX-CNN`, zum Beispiel `SRC-401`. Das Präfix sagt, wo es passiert ist, die erste Ziffer der Nummer, um welche Art es geht:

| Ziffer | Klasse |
|---|---|
| `1` | ungültige Eingabe |
| `2` | nicht gefunden |
| `3` | Konflikt |
| `4` | nicht erlaubt (Anmeldung, Recht, Token) |
| `5` | eine Grenze wurde überschritten |
| `6` | ein äußeres System ist ausgefallen (Netzwerk, Schlüsselspeicher, Dateisystem) |
| `9` | ein Programmfehler; bitte melden |

## Wo Sie einen Code sehen

- **Kommandozeile:** `error[SRC-401]: Text` auf der Standardfehlerausgabe, in Ihrer Sprache. Exit-Code 1 (2 bei ungültigen Argumenten).
- **Browser:** die Meldung des Bildschirms, mit dem Code.
- **REST:** `{ "code": "API-401", "description": "…" }` mit dem passenden HTTP-Status.
- **MCP:** ein Werkzeugergebnis mit `isError: true` und `code`, `description` (Englisch) und `params`.

Der Code ändert sich nie; der Text kann besser werden. Nennen Sie den Code, wenn Sie ein Problem melden.

## Der veröffentlichte Katalog

Die vollständige Liste der Codes mit Name, Schweregrad, ob ein Wiederholen hilft, der Version, seit der es ihn gibt, und einer Beschreibung wird aus dem Register `errors/error-codes.toml` erzeugt und in [`docs/error-codes.md`](../../error-codes.md) veröffentlicht (Englisch). Bearbeiten Sie sie nicht von Hand.

## Codes, denen Sie wahrscheinlich begegnen

| Code | Bedeutung | Was tun |
|---|---|---|
| `PRJ-101` | die Quelle ist weder ein vorhandener Ordner noch eine gültige Git-Adresse, oder die Adresse enthält Geheimnisse, oder der Ordner ist relativ | absoluten Pfad angeben; Geheimnisse aus der Adresse nehmen ([Quellen und Zugangsdaten](quellen-und-zugangsdaten.md)) |
| `PRJ-201` | kein solches Projekt | zuerst eintragen und scannen |
| `PRJ-302` | ein Scan dieses Projekts läuft schon | abwarten |
| `PRJ-303` | der Scan wurde abgebrochen, es gibt keinen neuen Snapshot | bei Bedarf neu starten |
| `SRC-201` | der Quellordner oder eine Datei existiert nicht oder ist nicht lesbar | Pfad und Rechte prüfen |
| `SRC-401`, `SEC-*` | Anmeldung an einer Git-Gegenstelle oder der Schlüsselspeicher ist fehlgeschlagen | siehe [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md#probleme) |
| `NET-601` | die Browser-Anmeldung von `credential login` wurde nicht abgeschlossen | erneut ausführen |
| `CLI-103` | es konnte kein Geheimnis gelesen werden, oder es war leer | per Pipe übergeben oder am Prompt eingeben |
| `SRC-501` | die Quelle oder eine Datei ist zu groß | `scan.max_source_mb` erhöhen oder Ordner ausschließen (`scan.excluded_folders`) |
| `PRS-201` | kein Sprachpaket für einen Dateityp | die Dateien dieses Typs werden nicht analysiert; Paket installieren |
| `PRS-501` | eine Datei brauchte zu lange zum Parsen | `scan.parse_timeout_ms` erhöhen, wenn die Datei in Ordnung ist |
| `CFG-101`, `CFG-102`, `CFG-103` | ungültiger Wert, nicht lesbare Datei, unbekannter Schlüssel in der Konfiguration | die in der Meldung genannte Einstellung korrigieren |
| `RES-*` | Ebene 2 (Indexer): Werkzeug fehlt oder zu alt (`RES-201`), Index zu groß, Zeitlimit, Ausgabe- oder Speichergrenze (`RES-501` bis `RES-504`), Indexer fehlgeschlagen (`RES-601`), Index beschädigt (`RES-101`), zu wenige Vorkommen zuordenbar (`RES-302`); Ebene 1 bleibt in jedem Fall | siehe [Sprach-Indexer](indexer.md#fehler) |
| `SEC-301`, `SEC-403` | Ebene 2 ist nicht eingeschaltet und bestätigt, oder der Aufrufer darf sie nicht bestätigen | einschalten und `codeskimr level2 confirm <Projekt>` in einem Terminal ausführen |
| `PRJ-303` | kein Fehler: Sie haben den Scan abgebrochen | erneut starten |
| `API-301` | der Port ist belegt | `--set server.port=…` |
| `API-401` | das Installations-Token fehlt oder ist falsch | für den Browser `codeskimr open` nutzen |
| `API-402` | `Host` oder `Origin` gehört nicht zu diesem Server | die Oberfläche unter `http://127.0.0.1:<Port>` oder `localhost` öffnen |
| `API-601` | der Befehl braucht einen laufenden Server | `codeskimr serve` starten |
| `AUTH-401` | dem Aufrufer fehlt das Recht | im Einzelnutzer-Modus sollte das nicht vorkommen; melden |
| `MCP-101` | ungültige Werkzeugargumente; `params.detail` nennt das betroffene | den Aufruf korrigieren |
| `MCP-202` | Symbol nicht gefunden | die ID aus `search_symbols` nehmen |
| `MCP-203` | das Projekt hat keinen Snapshot | scannen |
| `MCP-501` | kein Fehler: das Ergebnis wurde auf das Limit gekürzt | weniger verlangen oder `limit` erhöhen |
| `EXP-201`, `EXP-601` | die Export-Seite fehlt, oder die Exportdatei kann nicht geschrieben werden | siehe [HTML-Export](export.md#fehler) |
| `STO-501` | Datenträger voll | Platz schaffen |
| `CLI-101` | ungültige Kommandozeilenargumente | siehe `codeskimr --help` |
| `*-901` | ein unerwarteter interner Fehler | mit dem Code melden |

Konventionsbefunde (`CHK-101` bis `CHK-105`) sind Ergebnisse von `codeskimr check`, keine Fehler des Programms: ein Literal, ein Duplikat, ein Schichtverstoß, eine zu lange oder zu komplexe Funktion ([`docs/conventions.md`](../../conventions.md)).
