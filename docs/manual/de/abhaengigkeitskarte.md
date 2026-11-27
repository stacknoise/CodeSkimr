# Abhängigkeitskarte

Die Abhängigkeitskarte zeigt die **Schichten** des aktiven Projekts als Kästen und wie sie voneinander abhängen. Sie beantwortet: Hält sich das Programm an seine Architektur, und wo nicht?

Sie braucht Schichten. Legen Sie sie in der Oberfläche fest (der Hinweis auf der leeren Karte führt dorthin) oder in `.codeskimr.toml` ([Projekte und Scan](projekte-und-scan.md#schichten)) und scannen Sie neu. Ohne Schichten steht auf dem Bildschirm „Keine Schichten zu zeigen“.

## Die Karte lesen

- Ein Kasten je Schicht, eine Zeile je Schicht, äußere Schichten oben (die Reihenfolge Ihrer `[[layers]]`).
- Eine Linie zwischen zwei Kästen heißt „nutzt“. Ihre Beschriftung zählt die Verweise dahinter.
- Linien gegen die festgelegte Schichtung sind in der Verstoßfarbe markiert, und die Legende zeigt, wie viele es sind. Ein Hinweis oben listet die Verstöße („Anzeigen“ wählt sie aus). Linien, die zu einem Zyklus gehören, sind ebenfalls markiert.

Das Layout berechnet CodeSkimr selbst (geschichtetes Zeichnen), sodass dasselbe Projekt immer dasselbe Bild ergibt.

## Bedienung

- Das Mausrad zoomt; Ziehen mit der linken Maustaste auf dem freien Grund (oder mit der Mittelmaustaste oder gehaltener `Leertaste`) verschiebt; `F` (oder der Knopf „Einpassen“) passt die Karte ins Fenster ein.
- Ein Klick auf einen Kasten wählt ihn. Seine Linien sind nach Richtung gefärbt („Nutzt“ und „Genutzt von“), alle übrigen werden schwach. Das Seitenfeld zeigt die Schicht, ihre Zahl der Verweise, was sie **nutzt** und wovon sie **genutzt wird**, jeweils mit der Anzahl.
- Die Ebene „Schichten“ (Vorgabe) oder „Namespaces“ in der Werkzeugleiste lädt die Karte neu; die Leiste bleibt auch ohne Schichten, damit die anderen Ebenen wählbar sind. Bei nur einer Schicht steht der Hinweis, dass eine Abhängigkeitskarte mindestens zwei braucht. Die Namespace-Ebene ist wie der Typ-Fokus aufgebaut: ein Namensraum in der Mitte (zuerst der mit den meisten Verweisen), oben die Namensräume, die ihn nutzen, unten die, die er nutzt; ein Klick wählt eine Kachel und zeigt rechts die Details, ein Doppelklick macht sie zur Mitte. Der Name wird an den Punkten geteilt: die Kachel zeigt den letzten Teil, der Rest steht klein als Pfad darunter. Mehr als 24 Nachbarn je Seite werden gekürzt und vermerkt; lange Reihen brechen in mehrere Zeilen um (`graph.row_boxes`). „Typ-Fokus“ zeigt einen Typ in der Mitte (Typname in der Kachel, darunter klein die Namensräume als Pfad, die Reihen zentriert), oben die Typen, die ihn nutzen, unten die Typen, die er nutzt; es ist verfügbar, wenn im Explorer ein Typ gewählt ist. Mehr als 12 Typen je Seite werden gekürzt und vermerkt.
- Der Schalter „alle Verbindungen schwach“ zeigt oder verbirgt die Linien, die nicht zur Auswahl gehören; „nur Verstöße“ verbirgt jede Linie, die die Schichtung nicht verletzt.
- Kästen, die mit dem gewählten nicht verbunden sind, treten zurück (sie werden blass; Darüberfahren oder Fokus macht einen wieder voll sichtbar).
- `Enter` bei gewählter Schicht öffnet den Explorer für das Projekt.

## Module

Die Ebene „Module“ in der Werkzeugleiste zeigt die Namespaces des Projekts als Module, ein Kasten je Modul in der Zeile seiner Schicht (ADR 0108). Das Feld rechts hat eine Tabelle und drei Listen:

- **Tabelle.** Je Modul: an wie vielen Stellen in anderen Modulen es genutzt wird (`ein`), wie viele Stellen darin andere Module nutzen (`aus`), seine **Stabilität** `aus / (ein + aus)` und die Zahl seiner Typen. Stabilität 0 heißt: viele hängen vom Modul ab und es hängt von wenig ab, es sollte sich selten ändern; 1 heißt das Gegenteil. Ein Modul ohne jede Abhängigkeit hat keine. Gezählt werden Stellen im Code, nicht verschiedene Module oder Typen.
- **Zyklen.** Module, die sich im Ring voneinander abhängen. Zu jedem Zyklus steht, welche Abhängigkeiten man wegnimmt, um ihn zu beenden, die mit den wenigsten Stellen zuerst. Das ist ein Vorschlag nach der Zahl der zu ändernden Stellen, keine Aussage darüber, welche Richtung richtig ist.
- **Gegen die Schichtung** und **Auffällig.** Abhängigkeiten zwischen Modulen, die die Schichten nicht erlauben, und Module, die an vielen Stellen genutzt werden und von einem deutlich weniger stabilen Modul abhängen.

Mehr Module als `graph.max_modules` werden nicht gezeichnet und nicht aufgelistet (die am meisten verwendeten bleiben); Zyklen und die beiden Listen zählen immer alle Module. Der Knopf „Die Stellen zeigen“ unter einer Abhängigkeit, die einen Zyklus beendet oder gegen die Schichtung geht, listet die Stellen dieser Abhängigkeit: Datei und Zeile, und welches Symbol des einen Moduls welches Symbol des anderen nutzt. Es sind höchstens `graph.max_places`; wie viele mehr es gibt, steht dabei. Dieselben Zahlen gibt es auf der Kommandozeile (`codeskimr modules`, `--json` für Skripte; `--from` und `--to` nennen eine Abhängigkeit und listen ihre Stellen) und für Agenten (`get_dependencies` mit `level` `module`; `from_module` und `to_module` nennen eine Abhängigkeit).

## Bekannte Grenzen

Die Karte zeigt nur Beziehungen zwischen Schichten. Eine Linie gilt als „gegen die Schichtung“, wenn die Quellschicht eine `may_use`-Liste hat, die die Zielschicht nicht nennt ([Projekte und Scan](projekte-und-scan.md#schichten)); eine Schicht ohne `may_use` ist nicht beschränkt. Nehmen Sie die Karte als Hinweis, nicht als Urteil.
