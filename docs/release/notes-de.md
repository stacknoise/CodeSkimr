# CodeSkimr 1.0.0

Die erste stabile Version. Sie ist für Windows (x86_64), Linux (x86_64) und macOS (Apple Silicon) gebaut.

## Installieren

Pro System eine einzelne Datei (Oberfläche und Sprachpakete stecken darin), dazu eine `.sha256`-Datei. Das Handbuch liegt im Repository unter `docs/manual/de/`.

**Windows (PowerShell):** `codeskimr-1.0.0-windows-x86_64.exe` herunterladen und starten: `.\codeskimr-1.0.0-windows-x86_64.exe`. Windows SmartScreen warnt, weil die Datei noch nicht signiert ist: „Weitere Informationen“, dann „Trotzdem ausführen“. Prüfsumme: `Get-FileHash .\codeskimr-1.0.0-windows-x86_64.exe -Algorithm SHA256` und mit der `.sha256`-Datei vergleichen.

**Linux:** `chmod +x codeskimr-1.0.0-linux-x86_64`, dann `./codeskimr-1.0.0-linux-x86_64`. Prüfsumme: `sha256sum -c codeskimr-1.0.0-linux-x86_64.sha256`. Linux braucht einen Secret Service (GNOME Keyring oder KWallet); ohne ihn bricht `codeskimr serve` mit `SEC-601` ab.

**macOS (Apple Silicon):** `chmod +x codeskimr-1.0.0-macos-aarch64`, dann `xattr -d com.apple.quarantine codeskimr-1.0.0-macos-aarch64` (nimmt die Sperre von Gatekeeper weg, die Datei ist noch nicht signiert) und `./codeskimr-1.0.0-macos-aarch64`. Prüfsumme: `shasum -a 256 -c codeskimr-1.0.0-macos-aarch64.sha256`.

Das Programm startet den lokalen Server und öffnet den Browser. **Dieses Fenster offen lassen, solange du CodeSkimr benutzt:** Schließt du es, endet der Server, und die Oberfläche im Browser zeigt „Nicht verbunden“. Herkunft prüfen (optional): `gh attestation verify <Datei> --repo stacknoise/CodeSkimr` zeigt, dass GitHub die Datei aus diesem Repository gebaut hat.

## Erste Schritte

Projekt anlegen und scannen: Handbuch unter `docs/manual/de/projekte-und-scan.md`. Die Daten liegen unter `%LOCALAPPDATA%\codeskimr` (Windows), `~/Library/Application Support/codeskimr` (macOS) bzw. `~/.local/share/codeskimr` (Linux); Löschen des Ordners setzt alles zurück.

## Bekannte Grenzen

<!-- ENTWURF: Die Aussage zur Genauigkeit ist Antons Entscheidung (Empfehlung: so nennen). Zahlen: docs/accuracy.md, docs/acceptance/level2.md; vor dem Veröffentlichen neu prüfen. -->
- **Verwendungen (Wo wird das benutzt?)** sind nicht überall compilergenau. Mit Stufe 2 (genaue Indexer) gemessen auf den Testkorpora: Rust, TypeScript und Java 99 % und mehr. **Python, C# und Kotlin liegen darunter und sind eine Näherung:** Python findet fast alles (Trefferquote 99 %), zeigt aber auch falsche Treffer (Genauigkeit rund 63 %); bei C# ordnet der Indexer scip-dotnet Überladungen falsch zu (Trefferquote 34 %, Genauigkeit 60 %, siehe `docs/accuracy.md`); Kotlin lässt sich nicht messen (kein Referenzwerkzeug).
- Stufe 2 startet fremde Werkzeuge im Projekt und braucht Ihre ausdrückliche Zustimmung; die Indexer installieren Sie selbst (Handbuch: `indexer.md`).
- Keine Signatur (Windows, macOS; kommt als Erstes nach 1.0) und kein Installationsprogramm: eine einzelne Datei je System.

Rückmeldungen bitte als Issue im Repository `stacknoise/CodeSkimr`.
