# Änderung

Die Ansicht „Änderung“ vergleicht zwei Scans des geöffneten Projekts und zeigt, was der Unterschied trifft. Sie beantwortet die Frage „Was bricht, wenn ich das übernehme?“. Der Eintrag in der Navigationsleiste ist verfügbar, sobald ein Projekt geöffnet ist.

![Was eine Änderung trifft](../../screenshots/change/impact-petrol.png)

## Welche Scans verglichen werden

CodeSkimr bewahrt die letzten Scans eines Projekts auf (standardmäßig fünf). Ein Scan ohne Änderung schreibt keinen Eintrag, jeder aufbewahrte Scan hat also etwas geändert. Die Ansicht vergleicht den neuesten Scan mit dem davor. Im Reiter „Vergleich“ lassen sich zwei beliebige aufbewahrte Scans wählen: Die Basis ist der ältere, das Ziel der neuere. Das Projekt braucht mindestens zwei Scans.

Der Reiter „Scan“ der Projektseite listet die aufbewahrten Scans. Der Knopf „Mit aktuellem vergleichen“ in einer Zeile öffnet diese Ansicht mit diesem Scan als Basis und dem neuesten als Ziel.

## Auswirkung

Fünf Kennzahlen: die geänderten Symbole (neu, geändert, entfernt), wie viele davon öffentlich sind, die Verwendungen, die sie treffen, die nicht angepassten Stellen und die Dateien, die sie enthalten. Die Tabelle listet jedes entfernte, geänderte oder neue Symbol mit der Art der Änderung, der Sichtbarkeit und dem Zustand seiner Verwendungen: „n offen“ oder „angepasst“. Wer eine Zeile wählt, mit der Maus oder den Pfeiltasten, sieht rechts, was sich geändert hat (die alte und die neue Signatur) und die direkten Verwendungen des Symbols, nach Datei gruppiert, mit der Sicherheit des Index („sicher“ oder „wahrscheinlich“).

Eine Verwendung ist „angepasst“, wenn das Symbol, das sie enthält, im selben Unterschied selbst geändert, verschoben oder entfernt wurde, sonst „nicht angefasst“. **Die nicht angefassten Verwendungen sind die Prüfliste**: Stellen, die das geänderte Symbol verwenden, aber nicht geändert wurden. „Nur offene“ zeigt genau diese, „Alle“ jede Verwendung. Ein Symbol, dessen Inhalt sich geändert hat, und ein neues Symbol haben keine Prüfliste, weil ihre Verwendungen durch diese Änderung nicht brechen können.

Unter der Tabelle nennt die Ansicht neue Schichtverstöße und neue Schichtzyklen (Abhängigkeiten zwischen Schichten, die der Ziel-Scan hat und der Basis-Scan nicht hatte) sowie die kritischen Symbole, die der Unterschied berührt.

Der Knopf „Als Markdown kopieren“ oben rechts kopiert die Auswirkung als Text für einen Kommentar zu einem Pull Request: die Zahlen, die Prüfliste der nicht angefassten Stellen, neue Schichtverstöße und Zyklen. Es ist der Text, den `codeskimr change --markdown` ausgibt, in der Sprache der Ansicht. Der Knopf meldet „Kopiert“ oder „Nicht kopiert“, wenn der Browser die Zwischenablage verweigert.

Bekannt sind nur direkte Verwendungen. Aufrufe über Reflection, in Zeichenketten oder aus anderen Repositories sieht der Vergleich nicht; die Ansicht sagt das. Wurde eine Liste auf ihre Grenze gekürzt (`compare.max_symbols`, `compare.max_usages` in der Konfiguration), sagt eine Zeile es.

## Vergleich

![Der Vergleich zweier Scans](../../screenshots/change/compare-petrol.png)

Die beiden Auswahlfelder wählen die Scans; „Vergleichen“ liest den Vergleich neu. Die vier Zahlen zählen die neuen, geänderten, entfernten und verschobenen Symbole. Die Liste zeigt jedes Symbol mit dem Grund (Signatur, Sichtbarkeit oder Inhalt; ein verschobenes Symbol zeigt seine alte und neue Datei). Zwei Symbole sind dasselbe Symbol, wenn Art, qualifizierter Name und Parametertypen gleich sind; ein umbenanntes Symbol erscheint deshalb als entfernt und neu.

## Auf der Kommandozeile

Dieselbe Antwort gibt es als `codeskimr scans`, `codeskimr compare` und `codeskimr change` (mit `--markdown` für einen Kommentar zu einem Pull Request; dafür hat die Ansicht den Knopf „Als Markdown kopieren“) und für Agenten als MCP-Werkzeug `impact_of_diff`. Siehe die Referenzen zur [Kommandozeile](../../cli.md) und zum [Vergleich](../../compare.md).

## Tasten

| Taste | Aktion |
|---|---|
| `←` `→` auf einem Reiter | vorheriger oder nächster Reiter |
| `↑` `↓`, `Pos1`, `Ende` in der Tabelle | vorheriges oder nächstes Symbol, erstes oder letztes wählen |
