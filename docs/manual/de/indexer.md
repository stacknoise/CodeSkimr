# Sprach-Indexer (Ebene 2)

Ebene 2 macht Verwendungen **exakt**: Statt zu raten, welches `Save()` ein Aufruf meint, übernimmt CodeSkimr die Antwort des Werkzeugs der Sprache, das die Typen kennt, und markiert diese Verwendungen als **geprüft**. Dieses Kapitel erklärt, was Sie installieren, wie Sie es einschalten und was beim Scan passiert.

> **Stand:** Ebene 2 ist für C#, TypeScript, JavaScript, Python, Rust, Java und Kotlin gebaut und standardmäßig **aus**. Sie wurde an Beispielprojekten gemessen (`docs/accuracy.md`), ist aber noch nicht formal abgenommen, C# insbesondere. Sehen Sie geprüfte Verwendungen als deutlich besser als ungeprüfte, nicht als Garantie. Der Entscheidungstext, ADR 0033, hat noch den Status „proposed“, Einzelheiten können sich ändern.

## Wozu

Ohne Typinformation ist das Finden der Verwendungen eines Namens Raten. CodeSkimr übernimmt deshalb das Ergebnis eines Indexers auf Compiler-Niveau (einen SCIP-Index), statt einen Compiler nachzubauen. Für jede Datei, die der Index abdeckt, ersetzen seine Verweise die Vermutungen und sind als **geprüft** markiert. Dateien ohne Index behalten das namensbasierte Ergebnis, markiert als **ungeprüft** ([Verwendungen](verwendungen.md#verlässlichkeit-geprüft-ungeprüft-vermutet)).

## Sie installieren die Indexer, CodeSkimr prüft sie

CodeSkimr **lädt nie einen Indexer herunter oder installiert ihn** und führt nie ein Paket aus dem Netz aus (kein `npx`). Sie installieren den Indexer für jede Sprache, die Sie nutzen, so wie Sie den Compiler installieren. Der Grund: Ein Indexer braucht ohnehin die Abhängigkeiten Ihres Projekts und ein passendes SDK (Paketwiederherstellung, `node_modules`, eine Python-Umgebung, den Gradle- oder Maven-Build), und die kann nur Sie bereitstellen.

| Sprache | Indexer | Zusätzlich nötig |
|---|---|---|
| C# | `scip-dotnet` (`dotnet tool install --global scip-dotnet`) | das .NET SDK, das das Projekt verlangt (`global.json`), wiederhergestellte Pakete |
| TypeScript, JavaScript | `scip-typescript` (`npm install --global @sourcegraph/scip-typescript`) | Node.js (`.nvmrc`), installierte `node_modules` |
| Python | `scip-python` (`npm install --global @sourcegraph/scip-python`) | Node.js, die Python-Umgebung des Projekts |
| Rust | `rust-analyzer` (`rustup component add rust-analyzer`) | Cargo und die Werkzeugkette, die das Projekt verlangt (`rust-toolchain.toml`) |
| Java, Kotlin | `scip-java` (siehe seine Installationsanleitung) | ein JDK, der Maven- oder Gradle-Build des Projekts |

Go, C, C++, PHP und Ruby (Basis-Stufe) haben keine Ebene 2; ihre Verwendungen bleiben ungeprüft. Jeder Indexer hat eine Mindestversion, die die Prüfung unten meldet.

Der Indexer wird im `PATH` Ihres Benutzers gesucht. Ordner innerhalb des Projekts werden aus dem `PATH` entfernt, ein Repository kann sich also kein eigenes `dotnet` oder `node` unterschieben. Eine virtuelle Umgebung im Projekt (`.venv`) wird deshalb für `scip-python` **nicht** gefunden: Aktivieren Sie sie außerhalb des Projekts oder installieren Sie die Werkzeuge global.

## Was CodeSkimr Ihnen zeigt: die Werkzeugprüfung

Immer wenn ein Projekt geladen wird (hinzugefügt, vollständig gescannt, wieder geöffnet), prüft CodeSkimr je Sprache, ob der Indexer und seine Werkzeugkette vorhanden sind und passen. Nichts wird installiert, und es läuft kein Projektcode. Das Ergebnis wird beim Projekt gespeichert und angezeigt:

- im **Projekte-Bildschirm**: eine Warnung mit jedem fehlenden Werkzeug, wofür es gebraucht wird und wie man es installiert, den Sprachen, die bereit sind, einem Knopf „Erneut prüfen“ und in der Tabelle einer Markierung „N Werkzeuge fehlen“;
- in der Zusammenfassung von `codeskimr scan` (und mit `--json` als `tools`);
- über MCP mit dem Werkzeug `get_project`.

Ein Werkzeug ist `ready` (bereit), `missing` (nicht im `PATH`), `too_old` (unter der Mindestversion oder der Version, die das Projekt in `global.json`, `.nvmrc` oder `rust-toolchain.toml` verlangt) oder `version_unknown` (es lief, aber eine Version war nicht lesbar). Ein fehlendes Werkzeug **stoppt nie einen Scan**: Sie bekommen Ebene 1, ungeprüft markiert. Ein schneller Neuscan nach einer Dateiänderung wiederholt die Prüfung nicht.

## Einschalten

Einen Indexer auszuführen heißt, Build-Werkzeuge Ihres Projekts auszuführen, also Projektcode. Deshalb ist es opt-in, in zwei Schritten, und nur Sie, der lokale Eigentümer, können sie ausführen:

1. **Einschalten** in Ihrer Benutzerkonfiguration (`config.toml`):

   ```toml
   [level2]
   enabled = true
   ```

2. **Einmal je Projekt bestätigen:**

   ```
   codeskimr level2 confirm <Projekt>
   ```

   `<Projekt>` ist die Projekt-ID oder der Ordner oder die Git-Adresse, aus der es gescannt wurde. Der Befehl braucht ein Terminal. Er gibt aus, was Ebene 2 ausführt und mit welchen Rechten (es kann lesen, was Sie lesen können, und das Netzwerk nutzen, es gibt keine Sandbox), zeigt Name, Ort und Branch des Projekts und startet nur, wenn Sie den **Namen des Projekts** eintippen. Ohne Terminal scheitert er mit `SEC-403`.

   `codeskimr level2 revoke <Projekt>` verbietet es sofort wieder.

Die Bestätigung ist an Name, Ort und Branch des Projekts gebunden, **nicht an einen Commit**: Wird das Projekt auf einen anderen Ordner oder ein anderes Repository gelegt, ist es wieder unbestätigt, ein `git pull` am selben Ort bleibt aber bestätigt. Eine Einstellung `level2` in der `.codeskimr.toml` des Projekts selbst wird **ignoriert**, damit ein gescanntes Repository weder einschalten kann, dass seine eigenen Build-Skripte laufen, noch eine Grenze anheben.

## Wann es läuft

- **`codeskimr scan`** führt Ebene 2 aus, nachdem Ebene 1 gespeichert ist: Je Sprache mit Indexer prüft es Erlaubnis und Werkzeuge, führt den Indexer aus, importiert den Index und ersetzt die Verweise je Datei. Es gibt eine Zeile je Sprache aus: „Level 2: … geprüft“ oder „übersprungen (Grund)“, zum Beispiel nicht bestätigt, Werkzeuge fehlen, Dateien geändert, abgebrochen oder ein Fehlercode.
- Der Watch-Modus (`--watch`) führt Ebene 2 nur mit seinem **ersten** Scan aus; spätere Neuscans sind Ebene 1.
- **Scans aus dem Browser oder über MCP führen Ebene 2 nie aus**, auch nicht nach Ihrer Bestätigung. MCP, REST-Token und der Browser können die Build-Werkzeuge eines Projekts nicht starten.
- Ebene 1 ist vollständig und nutzbar, bevor Ebene 2 startet; Browser und MCP warten nicht auf die Indexer.

Was auch schiefgeht: Der Scan ist fertig, und Ebene 1 bleibt. Ein fehlschlagender Indexer, ein Zeitlimit, ein beschädigter Index oder eine zu niedrige Zuordnung werden je Sprache gemeldet und brechen den Scan nie ab.

## Grenzen

Jeder Lauf ist begrenzt; ein Indexer, der eine Grenze überschreitet, wird samt allen Kindprozessen beendet, und Ebene 1 bleibt. Die Werte stehen in `config/default.toml`, Abschnitt `[level2]`:

| Einstellung | Vorgabe | Fehler |
|---|---|---|
| `timeout_seconds` | 900 | `RES-502` |
| `max_memory_mb` (ganzer Prozessbaum) | 4096 | `RES-504` |
| `max_output_bytes` (Konsole und Dateien) | 512 MiB | `RES-503` |
| `max_index_bytes` (größter gelesener Index) | 256 MiB | `RES-501` |
| `min_mapped` (Anteil der Projekt-Vorkommen, der zuordenbar sein muss) | 0,5 | `RES-302` |

Der Indexer bekommt eine geleerte Umgebung mit Positivliste (kein Token und kein Geheimnis von CodeSkimr erreicht ihn), läuft in einer eigenen Prozessgruppe, und seine Ausgabe wird geschwärzt, bevor sie protokolliert wird. **Das Netzwerk ist nicht gesperrt**, weil die Paketwiederherstellung es braucht; das ist das wichtigste Restrisiko und der Grund für die Bestätigung.

## Nach einer Änderung

Ein Index gilt für eine Datei nur, solange ihr Inhalt der zum Zeitpunkt der Indizierung ist. Nach einem Lauf vergleicht CodeSkimr die Dateien noch einmal mit der Platte; Dateien, die Sie inzwischen geändert haben, behalten Ebene 1 und werden als geändert gemeldet. Eine Datei, die Sie später bearbeiten, fällt auf Ebene 1 (ungeprüft) zurück, bis Ebene 2 wieder läuft, mit dem nächsten `codeskimr scan`.

## Fehler

`RES-201` Indexer oder Werkzeugkette fehlt oder ist zu alt, `RES-501` Index zu groß, `RES-502` Zeitlimit, `RES-503` Ausgabe zu groß, `RES-504` Speichergrenze, `RES-601` der Indexer konnte nicht gestartet werden oder ist fehlgeschlagen, `RES-101` der Index ist beschädigt, `RES-302` zu wenige Vorkommen lassen sich Ihren Symbolen zuordnen, `SEC-301` Ebene 2 ist nicht eingeschaltet und bestätigt, `SEC-403` der Aufrufer darf Indexer nicht bestätigen oder starten, `PRJ-303` der Scan wurde abgebrochen. In allen bleibt Ebene 1 bestehen. Siehe [Fehlercodes](fehler.md).
