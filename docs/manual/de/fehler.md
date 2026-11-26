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
- **Browser:** die Meldung des Bildschirms, mit dem Code in kleiner Schrift und einem Knopf „Details kopieren“, der Code und Meldung in die Zwischenablage legt, bereit für eine Fehlermeldung an uns.
- **REST:** `{ "code": "API-401", "description": "…" }` mit dem passenden HTTP-Status.
- **MCP:** ein Werkzeugergebnis mit `isError: true` und `code`, `description` (Englisch) und `params`.

Der Code ändert sich nie; der Text kann besser werden. Nennen Sie den Code, wenn Sie ein Problem melden.

## Der veröffentlichte Katalog

Auf der Kommandozeile listet `codeskimr codes` alle Codes mit ihren Meldungen auf, und `codeskimr codes SRC-401` erklärt einen: Meldung, Entwicklerbeschreibung, Gruppe, Schweregrad, ob ein neuer Versuch helfen kann und seit welcher Version es ihn gibt. Dasselbe gibt es in der App: Unter „Einstellungen“ öffnet „Fehlercodes anzeigen“ die Liste zum Nachschlagen, mit Suche, Filter nach Schweregrad und Gruppe und der Erklärung zu jedem Code.

Die vollständige Liste der Codes mit Name, Schweregrad, ob ein Wiederholen hilft, der Version, seit der es ihn gibt, und einer Beschreibung wird aus dem Register `errors/error-codes.toml` erzeugt und in [`docs/error-codes.md`](../../error-codes.md) veröffentlicht (Englisch). Bearbeiten Sie sie nicht von Hand.

## Codes, denen Sie wahrscheinlich begegnen

| Code | Bedeutung | Was tun |
|---|---|---|
| `APP-201` | der Datenordner kann nicht angelegt oder geöffnet werden | Pfad und Rechte prüfen (`--data-dir`) |
| `APP-202` | die private Datei zum Starten des Browsers konnte nicht angelegt werden; wird auch abgelehnt, wenn ihr Ordner ein Link oder eine Junction ist oder wenn ein Ordner darüber einem fremden Konto gehört (Windows; kann Firmenprofile auf Netzlaufwerken und Ordner auf einem zweiten Laufwerk treffen) | Rechte des Datenordners prüfen; unter Windows nennt das Protokoll (`codeskimr.log`, Ereignis `launch_parent_refused`) den abgelehnten Ordner: den Datenordner mit `--data-dir` an einen Ort legen, dessen Ordner alle dem eigenen Konto gehören; `codeskimr serve` starten und die Adresse von Hand öffnen |
| `APP-601` | das System hat keinen Browser gestartet | die von `codeskimr serve` ausgegebene Adresse von Hand öffnen |
| `APP-603` | das Programm konnte sich nicht im Hintergrund starten oder sein Tray-Symbol nicht anzeigen | mit `--console` starten, dann bleibt es im Fenster; das Log liegt im Datenordner (`codeskimr.log`) |
| `APP-604` | das Konsolenfenster für die Bestätigung von Ebene 2 ließ sich nicht starten | den angezeigten Befehl kopieren und in einem Terminal ausführen |
| `APP-602` | die im Programm enthaltenen Sprachpakete konnten nicht in den Datenordner geschrieben werden | Platz schaffen und Rechte des Datenordners prüfen; ein Ordner `languages/` neben dem Programm (mit mindestens einem `<Sprache>/language.toml`) wird stattdessen genutzt ([Installation](installation.md#bauen)) |
| `PRJ-102` | der Anzeigename eines Projekts ist leer, enthält Steuer- oder unsichtbare Zeichen oder ist länger als 80 Zeichen | kürzeren Namen ohne Zeilenumbruch eingeben |
| `PRJ-103` | der Umfang eines Projekts nennt einen Ordner, der nicht im Projekt liegt (leer, absolut, mit Laufwerk oder mit `..`), eine Sprache mit ungültigem Namen oder mehr Einträge als `scan.max_scope_entries` | Ordner relativ zum Projektordner angeben, zum Beispiel `docs` oder `web/alt` |
| `PRJ-101` | die Quelle ist weder ein vorhandener Ordner noch eine gültige Git-Adresse, oder die Adresse enthält Geheimnisse oder Leerzeichen oder beginnt mit einem Bindestrich, oder der Branchname ist ungültig, oder der Ordner ist relativ | absoluten Pfad angeben; Geheimnisse aus der Adresse nehmen ([Quellen und Zugangsdaten](quellen-und-zugangsdaten.md)) |
| `PRJ-201` | kein solches Projekt | zuerst eintragen und scannen |
| `PRJ-302` | ein Scan dieses Projekts läuft schon (auch beim Entfernen des Projekts) | abwarten oder abbrechen |
| `PRJ-303` | der Scan wurde abgebrochen, es gibt keinen neuen Snapshot | bei Bedarf neu starten |
| `PRJ-501` | so viele Scans wie erlaubt (4) laufen gleichzeitig | warten, bis einer beendet ist |
| `PRJ-502` | der Bericht eines Scans (seine Stufe) konnte nicht gespeichert werden; der Scan selbst ist gespeichert | erneut scannen; freien Platz im Datenordner prüfen |
| `SRC-201` | ein Quellordner, eine Quelldatei oder ein Sprachpaket-Ordner existiert nicht oder ist nicht lesbar | Pfad und Rechte prüfen |
| `SRC-401`, `SEC-*` | Anmeldung an einer Git-Gegenstelle oder der Schlüsselspeicher ist fehlgeschlagen | siehe [Quellen und Zugangsdaten](quellen-und-zugangsdaten.md#probleme) |
| `NET-601` | die Browser-Anmeldung von `credential login` wurde nicht abgeschlossen | erneut ausführen |
| `CLI-103` | es konnte kein Geheimnis gelesen werden, oder es war leer | per Pipe übergeben oder am Prompt eingeben |
| `SRC-501` | die Quelle oder eine Datei ist zu groß | `scan.max_source_mb` erhöhen oder Ordner ausschließen (`scan.excluded_folders`) |
| `PRS-201` | kein Sprachpaket für einen Dateityp | die Dateien dieses Typs werden nicht analysiert; Paket installieren |
| `PRS-501` | eine Datei brauchte zu lange zum Parsen und wurde ausgelassen; der Scan läuft weiter | `scan.parse_timeout_ms` erhöhen, wenn die Datei in Ordnung ist |
| `PRS-101` | eine Datei ist kein gültiges UTF-8 oder UTF-16 und fehlt im Ergebnis; der Scan läuft weiter. Gilt das für alle Dateien, scheitert der Scan mit diesem Code | die genannte Datei als UTF-8 speichern |
| `PRS-102` | eine Datei war während des Scans weg oder gesperrt und wurde ausgelassen; der Scan läuft weiter | erneut scannen, wenn die Datei wieder lesbar ist |
| `PRJ-503` | der Scan belegte mehr Arbeitsspeicher als `scan.max_memory_mb` (höchstens 70 % des eingebauten Speichers) und wurde angehalten; es wurde kein Stand gespeichert | Ordner ausschließen (`scan.excluded_folders`), `scan.max_candidates` senken oder die Grenze in der eigenen Konfigurationsdatei erhöhen |
| `RES-304` | ein Name trifft mehr Symbole als `scan.max_candidates` und nichts grenzt ihn ein; die Stelle wurde nicht zugeordnet (nur Warnung) | Stufe 2 aktivieren oder `scan.max_candidates` erhöhen (eigene Konfigurationsdatei) |
| `CFG-101`, `CFG-102`, `CFG-103` | ungültiger Wert, nicht lesbare Datei, unbekannter Schlüssel in der Konfiguration | die in der Meldung genannte Einstellung korrigieren |
| `CFG-104` | die Datei `.codeskimr.toml` ließ sich beim Speichern der Schichten in der Oberfläche nicht schreiben | Projektordner beschreibbar machen oder die `[[layers]]`-Tabellen von Hand eintragen |
| `CFG-105` | Ebene 2 ist in der Benutzerkonfiguration nicht eingeschaltet | `[level2]` / `enabled = true` in die eigene `config.toml` schreiben; CodeSkimr tut das nie selbst |
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
| `EXP-501` | für dieses Projekt läuft schon ein HTML-Export | warten, bis er beendet ist |
| `EXP-502` | das Projekt hat mehr Verweisungen als `export.max_references` (Vorgabe 600000) | Grenze in der eigenen Konfigurationsdatei erhöhen (etwa 250 Byte je Verweisung) oder ein kleineres Projekt exportieren |
| `EXP-201`, `EXP-601` | die Export-Seite fehlt, oder die Exportdatei kann nicht geschrieben werden | siehe [HTML-Export](export.md#fehler) |
| `STO-501` | Datenträger voll | Platz schaffen |
| `CLI-101` | ungültige Kommandozeilenargumente | siehe `codeskimr --help` |
| `*-901` | ein unerwarteter interner Fehler | mit dem Code melden |

Konventionsbefunde (`CHK-101` bis `CHK-105`) sind Ergebnisse von `codeskimr check`, keine Fehler des Programms: ein Literal, ein Duplikat, ein Schichtverstoß, eine zu lange oder zu komplexe Funktion ([`docs/conventions.md`](../../conventions.md)).
