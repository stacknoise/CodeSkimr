# CodeSkimr 0.9.0-preview.1 (Vorabversion)

Erste Version zum Ausprobieren. **Das ist noch nicht 1.0.** Sie ist für Windows (x86_64), Linux (x86_64) und macOS (Apple Silicon) gebaut.

## Installieren

Pro System eine einzelne Datei (Oberfläche und Sprachpakete stecken darin), dazu eine `.sha256`-Datei. Das Handbuch liegt im Repository unter `docs/manual/de/`.

**Windows (PowerShell):** `codeskimr-0.9.0-preview.1-windows-x86_64.exe` herunterladen und starten: `.\codeskimr-0.9.0-preview.1-windows-x86_64.exe`. Windows SmartScreen warnt, weil die Datei noch nicht signiert ist: „Weitere Informationen“, dann „Trotzdem ausführen“. Prüfsumme: `Get-FileHash .\codeskimr-0.9.0-preview.1-windows-x86_64.exe -Algorithm SHA256` und mit der `.sha256`-Datei vergleichen.

**Linux:** `chmod +x codeskimr-0.9.0-preview.1-linux-x86_64`, dann `./codeskimr-0.9.0-preview.1-linux-x86_64`. Prüfsumme: `sha256sum -c codeskimr-0.9.0-preview.1-linux-x86_64.sha256`. Linux braucht einen Secret Service (GNOME Keyring oder KWallet); ohne ihn bricht `codeskimr serve` mit `SEC-601` ab.

**macOS (Apple Silicon):** `chmod +x codeskimr-0.9.0-preview.1-macos-aarch64`, dann `xattr -d com.apple.quarantine codeskimr-0.9.0-preview.1-macos-aarch64` (nimmt die Sperre von Gatekeeper weg, die Datei ist noch nicht signiert) und `./codeskimr-0.9.0-preview.1-macos-aarch64`. Prüfsumme: `shasum -a 256 -c codeskimr-0.9.0-preview.1-macos-aarch64.sha256`.

Das Programm startet den lokalen Server und öffnet den Browser. Herkunft prüfen (optional): `gh attestation verify <Datei> --repo stacknoise/CodeSkimr` zeigt, dass GitHub die Datei aus diesem Repository gebaut hat.

## Erste Schritte

Projekt anlegen und scannen: Handbuch unter `docs/manual/de/projekte-und-scan.md`. Die Daten liegen unter `%LOCALAPPDATA%\codeskimr` (Windows), `~/Library/Application Support/codeskimr` (macOS) bzw. `~/.local/share/codeskimr` (Linux); Löschen des Ordners setzt alles zurück.

## Bekannte Grenzen

- **Verwendungen (Wo wird das benutzt?)** sind nicht überall compilergenau. Gemessen auf den Testkorpora: Rust und TypeScript 100 %, Python 99 %, Java 88 %, C# 34 % (Stufe 2, Fehler in scip-dotnet). Für Sprachen unter 99 % bitte Verwendungen als Näherung lesen.
- Stufe 2 (genaue Indexer) startet fremde Werkzeuge im Projekt und braucht Ihre ausdrückliche Zustimmung; die Indexer installieren Sie selbst (Handbuch: `indexer.md`).
- Die Meilensteine M4/M5 sind noch nicht abgenommen; Oberfläche und Berichte können sich ändern. Datenbankformat kann sich bis 1.0 ändern: dann Datenordner löschen und neu scannen.
- Keine Signatur (Windows, macOS; kommt als Erstes nach 1.0) und kein Installationsprogramm: eine einzelne Datei je System.

Rückmeldungen bitte als Issue im Repository `stacknoise/CodeSkimr`.
