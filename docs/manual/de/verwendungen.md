# Verwendungen

Verwendungen beantworten zwei Fragen zu einem Symbol: **Wer nutzt es?** und **Was erreicht eine Änderung daran?** Sie sind die Grundlage für Planung und für die Beurteilung der Wirkung eines Umbaus; deshalb kommt es darauf an, wie weit man ihnen trauen kann. Lesen Sie dazu den Abschnitt zur Verlässlichkeit unten.

## Den Bildschirm öffnen

Wählen Sie im Explorer ein Member (oder einen Typ) und drücken `U`, klicken „Verwendungen“ oder klicken die Zahl der Verwendungen in der Membertabelle. Der Bildschirm zeigt die Verwendungen dieses Symbols.

## Werkzeugleiste

- **Richtung:** beide, wer ruft auf (eingehend), was wird aufgerufen (ausgehend).
- **Tiefe:** 1, 2, 3 oder alle. „Alle“ wird am konfigurierten Höchstwert gekappt (`mcp.max_depth`, Vorgabe 5).
- **Tests zeigen:** Verweise aus Testcode bleiben weg, solange Sie das nicht einschalten.
- **Ebene:** Es gibt nur „Symbol“. „Namensräume“ und „Typen“ werden ausgeschaltet gezeigt; sie gehören zur [Abhängigkeitskarte](abhaengigkeitskarte.md).

Jede Änderung liest die Verwendungen neu.

## Der Graph

Links eine Spalte je Schritt der Aufrufer, in der Mitte das Symbol im Fokus (gelber Ring), rechts eine Spalte je Schritt der Aufgerufenen. Eine Karte zeigt Typ- und Membername, Art, Schicht und die Zahl der Stellen. Ein Klick auf eine Karte macht dieses Symbol zum Fokus. Linien sind blau (Aufrufe), orange (was der Fokus nutzt), violett (Tests) und grau gestrichelt (vermutet). Je Spalte werden höchstens 30 Karten gezeichnet; die Überschrift sagt, wie viele weitere es gibt. Ein Verweis, der einen Zyklus schließt, wird aufgelistet, aber nicht rückwärts gezeichnet.

## Die Trefferliste

Eine Zeile je Stelle: Art (Aufruf, Lesen, Schreiben, Test usw.), das aufrufende Symbol, Datei und Zeile und die Verlässlichkeit. `F8` und `Umschalt+F8` springen durch die Treffer, `Enter` (oder Doppelklick) zeigt den gewählten Treffer im Explorer, oder das Symbol im Fokus, wenn keiner gewählt ist. Die Liste zeigt Datei und Zeile, nicht den Code selbst.

Das Seitenfeld zeigt Signatur, Zahlen (Stellen, Dateien, Schichten), die **Auswirkung** einer Änderung (erreichte Symbole, Schichten, öffentliche Schnittstellen) und den Knopf „Im Explorer öffnen“. Konstanten, Eigenschaften und Schnittstellen-Member funktionieren wie Methoden.

## Verlässlichkeit: geprüft, ungeprüft, vermutet

CodeSkimr findet Verwendungen auf zwei Wegen. **Ebene 1** löst Namen ohne Typinformation auf: schnell, braucht nichts Installiertes, kann aber Verwendungen übersehen und falsche enthalten. **Ebene 2** übernimmt das Ergebnis des Werkzeugs der Sprache, das die Typen kennt ([Sprach-Indexer](indexer.md)). Jeder Treffer sagt, woher er stammt:

| Markierung | Bedeutung |
|---|---|
| **geprüft** | Ebene 2 hat ihn geliefert; er ist so verlässlich wie das Werkzeug der Sprache |
| **ungeprüft** | nur per Name gefunden; **nicht** von einem Compiler oder Indexer geprüft, kann falsch sein |
| **vermutet** | Ebene 1 ist selbst unsicher (mehrere Symbole passen, oder der Aufruf läuft über eine Schnittstelle); ein ungeprüfter Treffer mit geringerem Wert |

Geprüfte Treffer stehen zuerst. Das Seitenfeld zählt die drei getrennt („N Stellen geprüft, M ungeprüft, davon K vermutet“).

**Ohne Indexer für das Projekt ist jeder Treffer ungeprüft**, und der Bildschirm sagt das: Verwendungen und Auswirkung sind vorläufig und können unvollständig sein oder falsche Treffer enthalten. Nehmen Sie das für Entscheidungen ernst: Eine fehlende Verwendung kann heißen, dass Sie Code brechen, von dem Sie nichts wussten. Eine leere Liste ohne Indexer schließt Verwendungen nicht aus.

## Dasselbe Wissen anderswo

MCP bietet Code-Agenten dieselben Daten mit derselben Unterscheidung ([MCP einrichten](mcp.md)). Der Lauf durch die Verwendungen folgt diesen Regeln: Ein Verweis wird einmal aufgelistet, ein früher erreichtes Symbol wird nicht erneut durchlaufen (Zyklen enden also), Verweise enden bei dem Symbol, das die Stelle enthält (die Verwendungen eines Typs enthalten nicht die seiner Member; fragen Sie die Member getrennt ab), und Verweise aus Tests zählen nur auf Wunsch, außer in der Auswirkung, die sie immer mitzählt.
