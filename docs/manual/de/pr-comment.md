# Kommentar im Pull Request

Eine GitHub Action schreibt in jeden Pull Request einen Kommentar: welche Symbole die Änderung berührt und welche Verwendungen sie nicht angefasst hat. Der Kommentar wird bei jedem Push aktualisiert; es entsteht kein zweiter. CodeSkimr speichert keinen Token und spricht nicht selbst mit GitHub; die Action nutzt den Token, den GitHub dem Workflow gibt.

Ab Version 1.1 (dem ersten Release mit `codeskimr change --markdown`).

```yaml
name: CodeSkimr
on: pull_request
permissions:
  contents: read
  pull-requests: write
jobs:
  change:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: stacknoise/CodeSkimr/actions/change-comment@v1.1.0
        with:
          version: 1.1.0
          language: de
```

Die Action führt keinen Code des Pull Requests aus: Stufe 2 bleibt aus. Die Action scannt erst den Basis-Stand, dann den Stand des Pull Requests und schreibt die Ausgabe von `codeskimr change --markdown` (die zwei neuesten Scans). Sie läuft auf Linux-Runnern. Eingaben: `version` (Release von CodeSkimr), `language` (`de` oder `en`), `fail-on` (`none`, `new-layer-violation` oder `new-cycle`; Standard `none`), `github-token`.

- **Forks.** Der Token eines Pull Requests aus einem Fork darf nicht kommentieren. Die Action schreibt den Text dann in die Zusammenfassung des Laufs. Dafür nicht auf `pull_request_target` wechseln: Das würde Code des Forks mit Schreibrechten ausführen.
- **Grenzen.** Bekannt sind nur direkte Verwendungen; Reflection, Zeichenketten und andere Repositories sieht CodeSkimr nicht. Der Kommentar sagt das.
- **Andere CI-Systeme** rufen `codeskimr scan .` zweimal auf (Basis, Stand) und `codeskimr change . --markdown` und posten den Text selbst.
