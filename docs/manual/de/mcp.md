# MCP einrichten

MCP (Model Context Protocol) lässt einen Code-Agenten wie Claude Code oder Cursor CodeSkimr nach Ihrem Code fragen: Symbole suchen, Signaturen lesen, Verwendungen finden, prüfen, ob es etwas Ähnliches schon gibt. Es gibt zwei Wege der Verbindung.

## 1. stdio (empfohlen)

Der Agent startet `codeskimr mcp` selbst und spricht darüber mit CodeSkimr über Standardeingabe und -ausgabe. Es bedient **ein Projekt**, die Aufrufe brauchen also kein Projektargument.

1. Scannen Sie das Projekt einmal ([Projekte und Scan](projekte-und-scan.md)). Ohne Scan bricht der Befehl mit `PRJ-201` (nicht eingetragen) oder `MCP-203` (eingetragen, aber kein Snapshot) ab.
2. Sagen Sie dem Agenten, wie er es startet. Allgemeine Konfiguration (Claude Desktop, Cursor und andere nutzen diese Form):

```json
{
  "mcpServers": {
    "codeskimr": {
      "command": "codeskimr",
      "args": ["mcp", "--project", "/pfad/zu/ihrem/projekt"]
    }
  }
}
```

Für Claude Code: `claude mcp add codeskimr -- codeskimr mcp --project /pfad/zu/ihrem/projekt`. Nehmen Sie den vollen Pfad von `codeskimr`, wenn es nicht im `PATH` liegt. Ohne `--project` gilt der aktuelle Ordner des Agenten. Auf der Standardausgabe steht nichts außer dem Protokoll; Fehler gehen auf die Standardfehlerausgabe.

## 2. HTTP unter `/mcp`

Die laufende App (`codeskimr serve`) bietet dieselben Werkzeuge unter `http://127.0.0.1:7421/mcp` an (Streamable HTTP, Antworten sind JSON, es wird keine Sitzung gehalten). Eine App bedient alle Projekte, deshalb **nennt jeder Aufruf sein Projekt** (Argument `project`; ein Aufruf ohne es ergibt `MCP-101`). Die Projekt-IDs sind die der Projektliste.

Jede Anfrage, auch ein reines Lesen, braucht das Installations-Token als `Authorization: Bearer <Token>`, weil die Antworten die Struktur Ihres Codes enthalten und nicht für jedes Programm auf dem Rechner lesbar sein dürfen. `Host` und `Origin` werden zuerst geprüft. Ein Client wird mit der URL und diesem Header eingerichtet.

> **Stand:** Das Token gibt kein Befehl aus. Wie es in die Konfiguration eines Agenten gelangt, gehört zur Installationsarbeit und ist noch nicht verfügbar. Nutzen Sie bis dahin stdio.

## Die Werkzeuge

Alle Werkzeuge lesen nur, außer `rescan` (und `get_project` führt die Werkzeugprüfung aus, einige kurze Prozesse). Jede Antwort ist strukturiertes JSON und nennt den Snapshot, auf dem sie beruht (`snapshot.id`, `snapshot.age_seconds`).

| Werkzeug | Wofür |
|---|---|
| `search_symbols` | Typen und Member nach Namen finden (`query`, optional `kind`, `limit`) |
| `get_symbol` | Signatur, Dokumentation, Sichtbarkeit, Zeilen, Schicht, Metriken und Member eines Symbols lesen (ID oder qualifizierter Name) |
| `find_usages` | wer das Symbol nutzt (`direction` `incoming`, Vorgabe) oder was es nutzt (`outgoing`); `depth`, `include_tests` |
| `impact_of_change` | welche Symbole, Schichten und öffentlichen Schnittstellen eine Änderung erreichen kann |
| `get_dependencies` | wovon ein Symbol, Namensraum oder eine Schicht abhängt und ob das die Schichtung verletzt |
| `get_architecture` | die Schichten und die Abhängigkeiten zwischen ihnen |
| `find_similar_code` | gibt es das schon? Ein `symbol` oder ein `snippet` mit seiner `language` angeben |
| `find_constants` | vorhandene Konstanten nach Namen finden, bevor ein Literal dazukommt |
| `check_conventions` | die Konventionsprüfungen über Dateien des Projekts ausführen |
| `get_project` | Sprachen des Projekts und die **Werkzeugprüfung** der Ebene 2: welche Indexer bereit, fehlend oder zu alt sind, mit Zweck und Installationshinweis |
| `rescan` | geänderte Dateien neu einlesen (inkrementell); braucht das Recht zum Scannen. Es startet nie Ebene 2 |

Gute Gewohnheiten für Agenten: `find_similar_code` und `find_constants` aufrufen, bevor neuer Code entsteht, und `check_conventions` vor einem Commit.

## Ergebnisse lesen

- **Grenzen.** `limit` ist 20, wenn nichts angegeben ist, und wird auf höchstens 100 begrenzt. Ein gekürztes Ergebnis trägt einen `notice` mit `MCP-501` und dem angewandten Limit; `total` sagt, wie viele es gibt.
- **Verlässlichkeit.** Jede Verwendung hat eine `confidence`, `certain` oder `probable`. Eine wahrscheinliche ist ein bester Tipp, den ein Agent prüfen sollte. Treffer der Ebene 1 sind nicht von einem Compiler geprüft; siehe [Verwendungen](verwendungen.md#verlässlichkeit-geprüft-ungeprüft-vermutet). Jede Verwendung und jedes Symbol der Auswirkung trägt zudem `verified` (nur wahr, wenn Ebene 2 es geliefert hat); die Antwort zählt `verified` und `unchecked` getrennt, listet geprüfte zuerst und setzt `preliminary` auf wahr, solange eine aufgelistete Verwendung ungeprüft ist. **Ein Agent darf eine vorläufige Antwort nicht als vollständig oder exakt behandeln** ([Sprach-Indexer](indexer.md)).
- **Unbekanntes Symbol** ergibt `MCP-202`, nie eine leere Liste: „keine Verwendungen“ und „kein solches Symbol“ sind verschiedene Antworten. Ein Name, den mehrere Überladungen teilen, antwortet mit der niedrigsten ID; nehmen Sie die ID aus `search_symbols`, um genau zu sein.
- **Fehler** sind normale Werkzeugergebnisse mit `isError: true` und `{ "code", "description", "params" }`. Die Beschreibung ist immer Englisch, für Agenten. Siehe [Fehlercodes](fehler.md).

## Sicherheit

Der Agent läuft als Sie, der lokale Eigentümer, und darf die Struktur des Projekts lesen. Quelltext zu lesen und einen Scan zu starten sind eigene Rechte, die die Werkzeuge prüfen. `check_conventions` wählt nur unter den Dateien der eigenen Liste des Projekts aus; ein Pfadargument wird nie geöffnet, `..` oder ein absoluter Pfad wählt also nichts aus.
