# Sprach-Indexer

> **Stand:** Die hier beschriebenen Indexer sind in ADR 0032 beschlossen und entstehen in eigenen Arbeitspaketen. Bis dahin ist jede Verwendung „ungeprüft“ ([Verwendungen](verwendungen.md#verlässlichkeit-geprüft-ungeprüft-vermutet)), und der Verwendungs-Bildschirm sagt, dass kein Indexer eingerichtet ist. Dieses Kapitel beschreibt, wie es funktioniert, sobald sie da sind, damit Sie wissen, was Sie installieren müssen.

## Wozu

Ohne Typinformation ist das Finden der Verwendungen eines Namens Raten: `Save()` kann in vierzig Klassen existieren. Ein Compiler oder Language Service weiß, welche gemeint ist. CodeSkimr übernimmt deshalb das Ergebnis eines solchen Werkzeugs (einen SCIP-Index), statt einen Compiler nachzubauen. Wo für eine Datei ein Index vorliegt, ersetzen seine Verweise die Vermutungen für diese Datei und sind als **geprüft** markiert.

## Sie installieren sie, CodeSkimr prüft sie

CodeSkimr **lädt nie einen Indexer herunter oder installiert ihn** und führt nie ein Paket aus dem Netz aus. Sie installieren den Indexer für jede Sprache, die Sie nutzen, so wie Sie den Compiler installieren. Der Grund: Der Indexer braucht ohnehin die Abhängigkeiten Ihres Projekts und ein passendes SDK (Paketwiederherstellung, `node_modules`, eine Python-Umgebung, den Gradle- oder Maven-Build), und die kann nur Sie bereitstellen.

| Sprache | Indexer (geplant) | Zusätzlich nötig |
|---|---|---|
| C# | scip-dotnet | ein passendes .NET SDK, wiederhergestellte Pakete |
| TypeScript, JavaScript | scip-typescript | Node.js, installierte `node_modules` |
| Python | scip-python | eine Python-Umgebung mit den Paketen des Projekts |
| Rust | `rust-analyzer scip` | die Rust-Werkzeugkette |
| Java, Kotlin | scip-java | ein JDK, das Build-Werkzeug des Projekts |

Go, C, C++, PHP und Ruby (Basis-Stufe) haben keine Ebene 2; ihre Verwendungen bleiben ungeprüft.

Die endgültige Liste und die Versionen legt der Spike zu ADR 0032 fest; diese Tabelle kann sich ändern.

## Was CodeSkimr Ihnen zeigt

Immer wenn ein Projekt geladen wird (hinzugefügt, gescannt, wieder geöffnet), prüft CodeSkimr je Sprache des Projekts, ob der Indexer und seine Werkzeugkette vorhanden sind und eine brauchbare Version haben. Das Ergebnis wird beim Projekt gespeichert und in der Oberfläche, in der Zusammenfassung von `codeskimr scan` und über MCP angezeigt:

- welche Sprachen Ebene 2 bereit haben, und
- zu jedem fehlenden Werkzeug sein Name, wofür es gebraucht wird und wie man es installiert.

Ein fehlendes Werkzeug **stoppt nie einen Scan**: Sie bekommen Ebene 1, jedes Ergebnis als ungeprüft markiert. Die Prüfung führt keinen Projektcode aus.

## Einschalten

Einen Indexer auszuführen heißt, Build-Werkzeuge Ihres Projekts auszuführen; deshalb ist es opt-in:

- Sie schalten es je Projekt in Ihrer **Benutzerkonfiguration** (oder in Ihrem eigenen Projekteintrag) ein und bestätigen es einmal.
- Eine Einstellung `level2` in der `.codeskimr.toml` des Projekts selbst wird **ignoriert**, damit ein gescanntes Repository nicht einschalten kann, dass seine eigenen Build-Skripte laufen.
- Nur Sie, der lokale Eigentümer, können einen Indexer bestätigen und starten. MCP, REST-Token, Watch-Modus und CI können es nicht.

Für jeden Lauf gelten Grenzen (Zeit, Speicher, Größe der Ausgabe und des Index); ein Indexer, der sie überschreitet, wird als Prozessgruppe beendet. Seine Umgebung enthält keine Zugangsdaten von CodeSkimr. Das Netzwerk ist in der ersten Version nicht gesperrt, weil die Paketwiederherstellung es braucht; das ist der Grund für die einmalige Bestätigung.

## Nach einer Änderung

Ein Index gilt für eine Datei, solange ihr Inhalts-Hash der zum Zeitpunkt der Indizierung ist. Eine Datei, die Sie geändert haben, fällt auf Ebene 1 zurück und gilt als ungeprüft, bis Ebene 2 wieder läuft, beim nächsten Scan oder auf Wunsch. Ein beschädigter oder zu großer Index wird als Fehler gemeldet, und Ebene 1 bleibt bestehen; der Scan bricht dadurch nie ab.
