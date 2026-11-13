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

Für ein Repository, das eine Anmeldung verlangt, legen Sie eine **Zugangsangabe** an: einen Benutzernamen und ein Geheimnis (ein Passwort oder ein Token). Sie wird im Schlüsselspeicher Ihres Betriebssystems abgelegt (Windows-Anmeldeinformationsverwaltung, macOS-Schlüsselbund, Linux Secret Service) unter dem Dienstnamen `codeskimr`. Das Projekt speichert nur den Namen der Zugangsangabe, nie das Geheimnis.

- Das Geheimnis wird in dem Moment aus dem Schlüsselspeicher gelesen, in dem eine Anfrage es braucht, an die Git-Anmeldung gereicht und danach aus dem Speicher gelöscht. Nichts wird zwischengespeichert, protokolliert oder in Datenbank oder Cache-Ordner geschrieben.
- Eine Zugangsangabe wird nur an den Server geschickt, auf den die Quelle zeigt. Nach einer Weiterleitung auf einen anderen Host wird nichts geschickt.
- Git weigert sich, Zugangsdaten über einfaches `http://` zu schicken; nehmen Sie `https://` oder SSH.
- Ohne Zugangsangabe entscheidet der Credential-Helper Ihres Systems, wie bei `git` selbst.

> **Stand:** Oberfläche und Kommandozeile zum Anlegen, Ändern und Löschen von Zugangsangaben kommen mit dem Arbeitspaket WP-73 (Issue #160). Bis dahin speichert kein Befehl eine Zugangsangabe, und ein privates Repository antwortet mit `SRC-401`. Öffentliche Repositories und Ordner funktionieren ohne. Dieses Kapitel beschreibt die Schritte, sobald WP-73 eingebaut ist.

## Installations-Token

Ein zweites Geheimnis gibt es: das **Installations-Token** (`csk_` und 64 Hexziffern). Es schützt alles, was Zustand ändert oder Quelltext liefert, und den MCP-Endpunkt, damit andere Programme auf Ihrem Rechner den lokalen Server nicht nutzen können. Es entsteht beim ersten `codeskimr serve`, liegt im selben Schlüsselspeicher (Eintrag `installation-token`), wird von keinem Befehl angezeigt und nie protokolliert. `codeskimr open` gibt dem Browser statt des Tokens einen einmaligen Startcode (60 Sekunden gültig, einmal verwendbar); die Browser-Oberfläche behält das Token nur in ihrem Tab.

## Probleme

| Code | Was geschah | Was tun |
|---|---|---|
| `SRC-401` | die Gegenstelle verlangt eine Anmeldung, und keine ist verfügbar | Zugangsangabe anlegen ([Stand oben](#zugangsdaten)) oder den Credential-Helper des Systems prüfen |
| `SEC-201` | die genannte Zugangsangabe ist nicht im Schlüsselspeicher | neu anlegen |
| `SEC-401` | die Gegenstelle hat die Zugangsangabe abgelehnt | Benutzername und Geheimnis prüfen; Tokens laufen ab |
| `SEC-402` | der Schlüsselspeicher hat den Zugriff verweigert | Zugriff in der Systemabfrage erlauben |
| `SEC-601` | kein Schlüsselspeicher verfügbar | unter Linux einen Secret Service starten (zum Beispiel GNOME Keyring); `serve` braucht ihn |
| `SRC-202` | der Branch existiert auf der Gegenstelle nicht | den Namen bei `--branch` prüfen |
| `SRC-601` | Netzwerk- oder Gegenstellenfehler | später erneut versuchen; Verbindung prüfen |
