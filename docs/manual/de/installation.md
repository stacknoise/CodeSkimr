# Installation

## Stand der Auslieferung

CodeSkimr wird als **eine einzelne Datei je System** ausgeliefert (Windows x86_64, Linux x86_64, macOS Apple Silicon): `codeskimr-<Version>-<System>` (unter Windows mit `.exe`). Oberfläche und Sprachpakete stecken in der Datei; es gibt keinen Installer. Sie finden die Dateien auf der Release-Seite des Repositorys `stacknoise/CodeSkimr`, jeweils mit einer `.sha256`-Datei. Alternativ bauen Sie CodeSkimr aus dem Quelltext (siehe unten).

### Herunterladen, prüfen, starten

1. Datei und `.sha256` herunterladen und die Prüfsumme vergleichen: Windows `Get-FileHash .\codeskimr-<Version>-windows-x86_64.exe -Algorithm SHA256`, Linux `sha256sum -c codeskimr-<Version>-linux-x86_64.sha256`, macOS `shasum -a 256 -c codeskimr-<Version>-macos-aarch64.sha256`.
2. Optional die Herkunft prüfen: `gh attestation verify <Datei> --repo stacknoise/CodeSkimr` zeigt, dass GitHub die Datei aus diesem Repository gebaut hat.
3. Starten. Linux und macOS: vorher `chmod +x <Datei>`. Dann `./codeskimr-<Version>-<System> serve`; unter Windows `.\codeskimr-<Version>-windows-x86_64.exe serve`. Sie können die Datei in `codeskimr` umbenennen und in einen Ordner Ihres Suchpfads legen.
   Welche Version läuft, zeigt `codeskimr --version` in der Konsole und der Bildschirm „Einstellungen“ unter der Überschrift.

Die Dateien sind **noch nicht signiert** (die Signierung kommt als Erstes nach 1.0). Deshalb warnt das Betriebssystem beim ersten Start:

- **Windows (SmartScreen):** „Der Computer wurde durch Windows geschützt“ → „Weitere Informationen“ → „Trotzdem ausführen“.
- **macOS (Gatekeeper):** Meldung, die Datei könne nicht geprüft werden. Einmal `xattr -d com.apple.quarantine <Datei>` im Terminal ausführen, dann starten (oder in den Systemeinstellungen unter „Datenschutz & Sicherheit“ „Trotzdem öffnen“ wählen).
- **Linux:** keine Warnung; es braucht einen Secret Service (siehe unten).

## Was Sie brauchen

- Die Rust-Werkzeugkette (stable, installiert mit [rustup](https://rustup.rs); das Repository legt den Kanal in `rust-toolchain.toml` fest).
- Node.js und npm, um die Browser-Oberfläche in `web/` zu bauen.
- Einen **Schlüsselspeicher** des Betriebssystems: Windows-Anmeldeinformationsverwaltung, macOS-Schlüsselbund oder unter Linux einen Secret Service (zum Beispiel GNOME Keyring oder KWallet). CodeSkimr legt dort sein Installations-Token und Ihre Git-Zugangsdaten ab. Ohne brauchbaren Schlüsselspeicher bricht `codeskimr serve` mit `SEC-601` ab; das betrifft vor allem Linux-Rechner ohne Oberfläche.
- Git ist **nicht** nötig: CodeSkimr holt Git-Quellen selbst.

## Bauen

```
git clone https://github.com/stacknoise/CodeSkimr
cd CodeSkimr
cd web && npm ci && npm run build && cd ..
cargo build --release --locked
```

Das Programm liegt in `target/release/codeskimr` (`codeskimr.exe` unter Windows). Die gebaute Oberfläche liegt in `web/dist` und wird beim Bauen ins Programm eingebettet, darum kommt `web/` zuerst. Node.js 22 (`.node-version`). Die Sprachpakete aus `languages/` sind ebenfalls im Programm eingebettet; beim ersten Start schreibt es sie in den Datenordner (`languages/<Kennung>/`). Ein Ordner `languages/` neben dem Programm hat Vorrang, wenn er mindestens ein `<Sprache>/language.toml` enthält (ein Ordner `languages/` im aktuellen Ordner wird nicht benutzt), ebenso die Option `--languages` oder `CODESKIMR_LANGUAGES_DIR`; so lassen sich Sprachpakete ändern, ohne neu zu bauen.

## Erster Start

```
codeskimr
```

startet den Server und öffnet den Browser; die Oberfläche ist Teil des Programms (vor `cargo build` `web/` bauen). Unter Windows und macOS zieht CodeSkimr in den Infobereich (Tray) um: Das Konsolenfenster schließt sich sofort, und das Symbol hat das Menü „CodeSkimr öffnen“, „Log anzeigen“ und „CodeSkimr beenden“ (das Log ist `codeskimr.log` im Datenordner). Mit `codeskimr --console` und unter Linux gilt: **Lassen Sie das Konsolenfenster offen, solange Sie CodeSkimr benutzen:** Das Programm läuft in diesem Fenster, und mit dem Fenster endet der Server. Die Oberfläche im Browser zeigt dann in der Statusleiste „Nicht verbunden“ und „Läuft CodeSkimr noch? Das Konsolenfenster nicht schließen.“; sie verbindet sich wieder, sobald Sie CodeSkimr neu gestartet haben und der Server antwortet. Öffnen Sie CodeSkimr danach mit einem neuen Start. Die zwei Schritte gehen auch einzeln:

```
codeskimr serve
```

in einem Terminal gibt `Listening on http://127.0.0.1:7421` aus und läuft bis Strg+C. In einem zweiten Terminal:

```
codeskimr open
```

öffnet den Browser. Wie es weitergeht, steht in [Projekte und Scan](projekte-und-scan.md). Beim ersten `serve` legt CodeSkimr das Installations-Token im Schlüsselspeicher an; Sie sehen oder tippen es nie.

Ist Port 7421 belegt, bricht der Server mit `API-301` ab; einen anderen Port stellen Sie mit `--set server.port=7500` ein. Die Adresse lässt sich nicht ändern: Der Server hört nur auf der Loopback-Adresse.

## Wo die Daten liegen

| Was | Wo |
|---|---|
| Datenbank `codeskimr.db` und Cache der Git-Quellen `git/` | Windows `%LOCALAPPDATA%\codeskimr`, macOS `~/Library/Application Support/codeskimr`, Linux `$XDG_DATA_HOME/codeskimr` oder `~/.local/share/codeskimr`; ändern mit `--data-dir` oder `CODESKIMR_DATA_DIR` |
| Benutzerkonfiguration `config.toml` (optional) | Windows `%APPDATA%\codeskimr`, macOS `~/Library/Application Support/codeskimr`, Linux `$XDG_CONFIG_HOME/codeskimr`; ändern mit `--config` oder `CODESKIMR_CONFIG` |
| Sprachpakete des Programms `languages/<Kennung>/` (beim ersten Start geschrieben, kann gelöscht werden) | im Datenordner |
| Theme und Sprache der Oberfläche `preferences.json` | im Datenordner |
| Installations-Token, Git-Zugangsdaten | Schlüsselspeicher des Betriebssystems, Dienstname `codeskimr` |

## Konfiguration

Einstellungen gelten in dieser Reihenfolge, später gewinnt: die ins Programm eingebauten Vorgaben, Ihre Benutzerkonfiguration, die Datei `.codeskimr.toml` im gescannten Ordner und `--set key=value` auf der Kommandozeile (der Wert ist TOML, zum Beispiel `--set scan.excluded_folders=["a","b"]`). Ein Schlüssel, den es nicht gibt, wird mit `CFG-103` abgelehnt, ein Wert außerhalb seines Bereichs mit `CFG-101`. Die vollständige Liste der Schlüssel mit Erklärungen ist `config/default.toml`.

## Deinstallieren

Löschen Sie das Programm, den Datenordner und, wenn Sie möchten, die Einträge namens `codeskimr` in Ihrem Schlüsselspeicher.
