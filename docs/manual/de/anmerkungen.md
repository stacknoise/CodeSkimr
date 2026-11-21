# Notizen, Aufgaben und kritische Markierungen

An einem Symbol (einem Typ oder einem Member) können drei Arten von **Anmerkungen** hängen. Sie liegen auf Ihrem Rechner neben dem Projekt und überstehen einen Scan: Gibt es das Symbol noch, bleiben seine Anmerkungen daran.

| Art | Wofür |
|---|---|
| **Kritische Markierung** | „dieses Symbol ist systemkritisch“; eine Änderung daran löst nach dem nächsten Scan eine Meldung aus |
| **Notiz** | eine Bemerkung in einfachem Text |
| **Aufgabe** | Arbeit an einem Symbol verfolgen: Titel, optionaler Text, Status (offen, in Arbeit, erledigt), Fälligkeit und Zuständige oder Zuständiger |

Notizen werden als einfacher Text gespeichert. Schreiben Sie keine Passwörter oder Schlüssel hinein.

## Im Explorer

Wählen Sie einen Typ oder ein Member. Das Seitenfeld hat die Reiter **Info**, **Kritisch**, **Notizen** und **Aufgaben** (mit den Pfeiltasten wechseln). Drei Tasten wirken auf die Auswahl und sind auch Einträge der Schnellsuche (`Strg+K`):

- `K` markiert die Auswahl als kritisch oder nimmt die Markierung weg (mit einer optionalen Begründung).
- `N` legt eine Notiz an.
- `T` legt eine Aufgabe an.

Kritische Symbole haben in der Member-Tabelle einen Streifen und die Beschriftung „Kritisch“, im Baum ein Schild (eine zugeklappte Gruppe zeigt, wie viele kritische Symbole darunter liegen) und in den Details Schild und Beschriftung. Kritisch wird nie nur über die Farbe gezeigt. Der Schalter „Anmerkungen einblenden“ setzt die Texte von Notizen und Aufgaben unter die Zeilen. Die [Abhängigkeitskarte](abhaengigkeitskarte.md) hat die Schalter „Kritische hervorheben“ und „nur Kritische und ihre Nachbarn“.

## Meldungen über Änderungen

Nach einem Scan vergleicht CodeSkimr jedes kritische Symbol mit seinem zuletzt bestätigten Stand. Die Glocke in der oberen Leiste zeigt, wie viele Meldungen offen sind. Es gibt drei Arten:

- **Geändert:** Inhalt, Signatur, Komplexität oder Zeilenzahl haben sich geändert.
- **Entfernt:** Das Symbol gibt es nicht mehr; die Meldung sagt, wie viele Stellen noch darauf verweisen.
- **Verwaist:** Das Ziel wurde nicht gefunden; die Meldung nennt das nächstliegende Symbol. „Neu zuordnen“ verschiebt die Anmerkung dorthin, „Markierung entfernen“ nimmt die Markierung weg.

Die Schublade der Glocke hat die Reiter Offen, Geändert, Entfernt (verwaiste Ziele stehen hier) und Erledigt. Das Kreuz entfernt eine Meldung, „Alle offenen Meldungen entfernen“ entfernt nach einer Rückfrage alle eines Reiters. Ein Klick springt zum Symbol im Explorer. `Esc` schließt die Schublade.

## Das Brett

Der Bildschirm **Anmerkungen** zeigt alle Aufgaben des Projekts, je Status eine Spalte; eine Aufgabe wechselt die Spalte über die Statusliste ihrer Karte. Die Filter sind „Meine“, „Nur kritische“ und „Fällig“. „Meine“ funktioniert, sobald Sie in diesem Browser eine Anmerkung angelegt haben, denn daran lernt CodeSkimr Ihren Namen. Eine zweite Ansicht listet die Notizen.

## Auf der Kommandozeile

Alle Befehle nehmen das Projekt als Id oder als Ordner bzw. Git-Adresse, aus der es gescannt wurde. `--json` gibt die Antwort als JSON aus; ohne gibt es eine Zeile je Eintrag. Listen sind seitenweise: `--limit` und `--offset`, und der Text sagt, wie es weitergeht.

```
codeskimr annotations list <Projekt> [--kind critical|note|task] [--status open|in_progress|done] [--symbol <Name> --symbol-kind <Art> [--param <Typ>]...] [--limit N] [--offset N] [--json]
codeskimr annotations add <Projekt> --kind critical|note|task --symbol <Name> --symbol-kind <Art> [--param <Typ>]... [--title <Titel>] [--text <Text>] [--status <Status>] [--due <Sekunden>] [--assignee <Name>] [--json]
codeskimr annotations update <Projekt> <Id> [--symbol <Name> --symbol-kind <Art>] [--title ...] [--text ...] [--status ...] [--due ...] [--assignee ...] [--clear-text] [--clear-due] [--clear-assignee] [--json]
codeskimr annotations remove <Projekt> <Id> [--json]
codeskimr alerts list <Projekt> [--state open|dismissed] [--kind changed|removed|orphaned]... [--limit N] [--offset N] [--json]
codeskimr alerts dismiss <Projekt> <Id> [--json]
codeskimr alerts dismiss-all <Projekt> [--kind changed|removed|orphaned]... --yes [--json]
```

- `--symbol` ist der qualifizierte Name, `--symbol-kind` seine Art (`class`, `method`, `namespace` und so weiter). Bei Überladungen wiederholen Sie `--param` je Parametertyp.
- `--title` gehört zu Aufgaben. `--due` ist ein Zeitpunkt in Sekunden seit dem Unix-Beginn (1.1.1970).
- `alerts list` zeigt die offenen Meldungen, außer Sie fragen mit `--state dismissed` nach den entfernten. `alerts dismiss-all` tut ohne `--yes` nichts.
- Exit-Code `0`, oder `1` mit dem Fehlercode auf der Standardfehlerausgabe ([Fehlercodes](fehler.md)).

Beispiel:

```
codeskimr annotations add . --kind task --symbol Shop.Orders.OrderService.Place --symbol-kind method --title "Rundung prüfen" --assignee anna
codeskimr alerts list . --kind changed
```

## Für Code-Agenten

Die MCP-Werkzeuge `list_annotations` und `get_alerts` lesen dieselben Daten ([MCP einrichten](mcp.md)). Agenten können Anmerkungen und Meldungen lesen, aber nicht ändern oder entfernen.

## Wenn ein Projekt entfernt wird

Mit dem Projekt werden auch seine Anmerkungen und Meldungen gelöscht ([Projekte und Scan](projekte-und-scan.md)). Der [HTML-Export](export.md) enthält sie nicht.
