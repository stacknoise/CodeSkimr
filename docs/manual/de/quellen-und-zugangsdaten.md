# Quellen und Zugangsdaten

## Quellen

| Quelle | Wie |
|---|---|
| Ordner | ein absoluter Pfad; wird an Ort und Stelle gescannt. Einen Watch-Modus gibt es (`codeskimr scan <Ordner> --watch`). |
| Git-Repository | eine Adresse (`https://…`, `ssh://…`, `git@host:org/repo` oder mit der Endung `.git`) und ein Branch (Vorgabe `main`). CodeSkimr legt einen flachen Klon (Tiefe 1) dieses einen Branches im eigenen Cache-Ordner `git/` im Datenordner an und scannt ihn. Ein zweiter Scan aktualisiert die Kopie; schlägt die Aktualisierung fehl, bleibt die alte Kopie. |

Eine Git-Quelle lässt sich nicht beobachten (`CLI-102`).

## Geheimnisse gehören nie in eine Adresse

Eine Adresse mit Benutzername und Passwort, einem Token, einer Query oder einem Fragment (zum Beispiel `https://user:token@host/repo` oder `…?token=abc`) wird **mit `PRJ-101` abgelehnt**, mit einer zweiten Zeile, die den Grund nennt. Die Datenbank würde sie speichern und das Programm ausgeben. Ein bloßer SSH-Benutzername (`git@host:org/repo`) ist in Ordnung. Cache-Ordner heißen nach einem Hash, die Adresse steht also auch in keinem Pfad und keinem Fehler.

## Zugangsdaten

Für ein Repository, das eine Anmeldung verlangt, legen Sie eine **Zugangsangabe** ab: einen Benutzernamen und ein Geheimnis (ein Passwort oder ein Token) unter einem Namen Ihrer Wahl, zum Beispiel `work-github`. Sie liegt im Schlüsselspeicher Ihres Betriebssystems (Windows-Anmeldeinformationsverwaltung, macOS-Schlüsselbund, Linux Secret Service) unter dem Dienstnamen `codeskimr`. Ein Projekt merkt sich nur den **Namen** seiner Zugangsangabe, nie das Geheimnis.

### Token oder Passwort ablegen

```
codeskimr credential set work-github --user anton
```

Das Geheimnis wird ohne Anzeige vom Terminal gelesen. Ohne Terminal wird es von der Standardeingabe gelesen, was sich für Skripte eignet:

```
printf %s "$TOKEN" | codeskimr credential set work-github --user anton
```

Es gibt absichtlich **keine Option für das Geheimnis**, damit es nie in Ihrem Shell-Verlauf, einer Prozessliste oder einem Umgebungsauszug landet. Ein leeres Geheimnis wird abgelehnt (`CLI-103`). Ein erneutes `set` mit demselben Namen ersetzt die Zugangsangabe. Geheimnisse von der Standardeingabe sind auf 64 KiB begrenzt.

### Mit GitHub oder GitLab anmelden (ohne Passwort)

```
codeskimr credential login work-github --provider github --client-id <id>
codeskimr credential login work-gitlab --provider gitlab --client-id <id> --host https://gitlab.example.com
```

Das ist der OAuth Device Flow: Der Befehl gibt eine Webseite und einen kurzen Code aus, Sie öffnen die Seite im Browser, geben den Code ein und bestätigen, und der Befehl speichert das Token. Sie brauchen die `--client-id` einer beim Anbieter registrierten OAuth-Anwendung; sie ist kein Geheimnis. `--host` wählt eine selbst betriebene Installation und muss eine HTTPS-Adresse sein. Das Token bekommt den kleinsten Bereich, den das Lesen braucht (GitHub `repo`, GitLab `read_repository`). Schließen Sie die Anmeldung im Browser nicht ab, scheitert der Befehl mit `NET-601`.

GitHub-Tokens einer OAuth-Anwendung laufen nicht ab. GitLab-Tokens laufen nach zwei Stunden ab, und CodeSkimr kann sie noch nicht erneuern: Führen Sie `credential login` erneut aus, wenn `SEC-401` erscheint.

### Zugangsangabe entfernen

```
codeskimr credential delete work-github
```

Ein Name, den es nicht gibt, ist kein Fehler. Der Name `installation-token` gehört dem lokalen Server und wird abgelehnt.

### Verwenden

```
codeskimr scan https://github.com/org/private-repo --credential work-github
```

`--credential` gilt nur für Git-Quellen; für einen Ordner ergibt es `PRJ-101`. Ohne `--credential` entscheidet der Credential-Helper Ihres Systems, wie bei `git` selbst. **Es gibt nie eine Eingabeaufforderung im Terminal**, ein Server oder eine MCP-Sitzung kann also nicht an einer Anmeldefrage hängen bleiben; ist keine Anmeldung verfügbar, scheitert der Scan mit `SRC-401`.

### Was CodeSkimr mit dem Geheimnis tut

- Es wird in dem Moment aus dem Schlüsselspeicher gelesen, in dem eine Anfrage es braucht, an die Git-Anmeldung gereicht und danach aus dem Speicher gelöscht. Nichts wird zwischengespeichert, protokolliert oder in Datenbank oder Cache-Ordner geschrieben.
- Es wird nur an den Server geschickt, auf den die Quelle zeigt. Nach einer Weiterleitung auf einen anderen Host wird nichts geschickt.
- Git weigert sich, Zugangsdaten über einfaches `http://` zu schicken; nehmen Sie `https://` oder SSH.

> **Stand:** Die Browser-Oberfläche hat noch kein Formular für Zugangsangaben; legen Sie sie auf der Kommandozeile an.

## Installations-Token

Ein zweites Geheimnis gibt es: das **Installations-Token** (`csk_` und 64 Hexziffern). Es schützt alles, was Zustand ändert oder Quelltext liefert, und den MCP-Endpunkt, damit andere Programme auf Ihrem Rechner den lokalen Server nicht nutzen können. Es entsteht beim ersten `codeskimr serve`, liegt im selben Schlüsselspeicher (Eintrag `installation-token`), wird von keinem Befehl angezeigt und nie protokolliert. `codeskimr open` gibt dem Browser statt des Tokens einen einmaligen Startcode (60 Sekunden gültig, einmal verwendbar); die Browser-Oberfläche behält das Token nur in ihrem Tab.

## Probleme

| Code | Was geschah | Was tun |
|---|---|---|
| `SRC-401` | die Gegenstelle verlangt eine Anmeldung, und keine ist verfügbar | Zugangsangabe anlegen ([siehe oben](#zugangsdaten)) und mit `--credential` angeben, oder den Credential-Helper des Systems prüfen |
| `SEC-201` | die genannte Zugangsangabe ist nicht im Schlüsselspeicher | mit `credential set` oder `credential login` anlegen |
| `SEC-202` | der Schlüsselspeicher-Eintrag ist beschädigt | mit `credential delete` löschen und neu anlegen |
| `NET-601` | die Anmeldung im Browser wurde nicht abgeschlossen | `credential login` wiederholen und rechtzeitig bestätigen |
| `CLI-103` | es konnte kein Geheimnis gelesen werden, oder es war leer | das Geheimnis per Pipe übergeben oder am Prompt eingeben |
| `SEC-401` | die Gegenstelle hat die Zugangsangabe abgelehnt (GitHub antwortet so auch bei ungültigem Token für ein verborgenes Repository) | Benutzername, Geheimnis und Rechte des Tokens prüfen; Tokens laufen ab (GitLab-OAuth-Tokens nach zwei Stunden) |
| `SEC-402` | der Schlüsselspeicher hat den Zugriff verweigert | Zugriff in der Systemabfrage erlauben |
| `SEC-601` | kein Schlüsselspeicher verfügbar | unter Linux einen Secret Service starten (zum Beispiel GNOME Keyring); `serve` braucht ihn |
| `SRC-202` | der Branch existiert auf der Gegenstelle nicht | den Namen bei `--branch` prüfen |
| `SRC-601` | Netzwerk- oder Gegenstellenfehler | später erneut versuchen; Verbindung prüfen |
