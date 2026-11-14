# Installation

## Stand der Auslieferung

Installationsprogramme und einzelne ausführbare Dateien für Windows, Linux und macOS gehören zum Release 1.0 (Meilenstein M7) und sind noch nicht veröffentlicht. Als Vorabversion gibt es fertige Pakete für Windows, Linux und macOS (Apple Silicon) (siehe Release-Seite im Repository, `LIES-MICH.md` im Paket). Sonst bauen Sie CodeSkimr aus dem Quelltext. Dieses Kapitel beschreibt diesen Weg; mit 1.0 wird es durch die Anleitung für den Installer ersetzt.

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

Das Programm liegt in `target/release/codeskimr` (`codeskimr.exe` unter Windows). Die gebaute Oberfläche liegt in `web/dist` und wird beim Bauen ins Programm eingebettet, darum kommt `web/` zuerst. Node.js 22 (`.node-version`). Die Sprachpakete werden aus dem Ordner `languages/` gelesen, zuerst neben dem Programm, dann im aktuellen Ordner (die Option `--languages` oder `CODESKIMR_LANGUAGES_DIR` nennt einen anderen Ordner). Sie ins Programm einzubetten, kommt mit dem Release-Paket.

## Erster Start

```
codeskimr
```

startet den Server und öffnet den Browser; die Oberfläche ist Teil des Programms (vor `cargo build` `web/` bauen). Die zwei Schritte gehen auch einzeln:

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
| Installations-Token, Git-Zugangsdaten | Schlüsselspeicher des Betriebssystems, Dienstname `codeskimr` |

## Konfiguration

Einstellungen gelten in dieser Reihenfolge, später gewinnt: die ins Programm eingebauten Vorgaben, Ihre Benutzerkonfiguration, die Datei `.codeskimr.toml` im gescannten Ordner und `--set key=value` auf der Kommandozeile (der Wert ist TOML, zum Beispiel `--set scan.excluded_folders=["a","b"]`). Ein Schlüssel, den es nicht gibt, wird mit `CFG-103` abgelehnt, ein Wert außerhalb seines Bereichs mit `CFG-101`. Die vollständige Liste der Schlüssel mit Erklärungen ist `config/default.toml`.

## Deinstallieren

Löschen Sie das Programm, den Datenordner und, wenn Sie möchten, die Einträge namens `codeskimr` in Ihrem Schlüsselspeicher.
