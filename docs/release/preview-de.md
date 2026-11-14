# CodeSkimr 0.9.0-preview.1 (Vorabversion)

Erste Version zum Ausprobieren. **Das ist noch nicht 1.0.** Sie ist für Windows (x86_64), Linux (x86_64) und macOS (Apple Silicon) gebaut.

## Installieren (Windows, PowerShell)

1. `codeskimr-0.9.0-preview.1-windows-x86_64.zip` herunterladen und entpacken, zum Beispiel nach `C:\Tools\codeskimr`.
2. Optional die Prüfsumme vergleichen: `Get-FileHash .\codeskimr-0.9.0-preview.1-windows-x86_64.zip -Algorithm SHA256` und mit der `.sha256`-Datei vergleichen.
3. Im entpackten Ordner starten:

```
.\codeskimr.exe
```

Das startet den lokalen Server und öffnet den Browser. Die Oberfläche steckt im Programm; der Ordner `languages\` muss neben `codeskimr.exe` bleiben. Windows SmartScreen kann warnen, weil das Programm noch nicht signiert ist: „Weitere Informationen“, dann „Trotzdem ausführen“.

## Installieren (Linux)

```
tar xzf codeskimr-0.9.0-preview.1-linux-x86_64.tar.gz
cd codeskimr-0.9.0-preview.1-linux-x86_64
./codeskimr
```

Linux braucht einen Secret Service (GNOME Keyring oder KWallet); ohne ihn bricht `codeskimr serve` mit `SEC-601` ab.

## Installieren (macOS, Apple Silicon)

```
tar xzf codeskimr-0.9.0-preview.1-macos-aarch64.tar.gz
cd codeskimr-0.9.0-preview.1-macos-aarch64
xattr -dr com.apple.quarantine .
./codeskimr
```

Das Programm ist noch nicht signiert; der Befehl `xattr` nimmt die Sperre von macOS Gatekeeper weg. Der Ordner `languages/` muss neben `codeskimr` bleiben.

## Erste Schritte

Projekt anlegen und scannen: Handbuch unter `docs/manual/de/projekte-und-scan.md`. Die Daten liegen unter `%LOCALAPPDATA%\codeskimr` (Windows) , `~/Library/Application Support/codeskimr` (macOS) bzw. `~/.local/share/codeskimr` (Linux); Löschen des Ordners setzt alles zurück.

## Bekannte Grenzen

- **Verwendungen (Wo wird das benutzt?)** sind nicht überall compilergenau. Gemessen auf den Testkorpora: Rust und TypeScript 100 %, Python 99 %, Java 88 %, C# 34 % (Stufe 2, Fehler in scip-dotnet). Für Sprachen unter 99 % bitte Verwendungen als Näherung lesen.
- Stufe 2 (genaue Indexer) startet fremde Werkzeuge im Projekt und braucht Ihre ausdrückliche Zustimmung; die Indexer installieren Sie selbst (Handbuch: `indexer.md`).
- Die Meilensteine M4/M5 sind noch nicht abgenommen; Oberfläche und Berichte können sich ändern. Datenbankformat kann sich bis 1.0 ändern: dann Datenordner löschen und neu scannen.
- Keine Signatur (Windows, macOS), kein Installationsprogramm (kommt mit 1.0).

Rückmeldungen bitte als Issue im Repository `stacknoise/CodeSkimr`.
