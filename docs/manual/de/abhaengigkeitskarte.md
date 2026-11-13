# Abhängigkeitskarte

Die Abhängigkeitskarte zeigt die **Schichten** des aktiven Projekts als Kästen und wie sie voneinander abhängen. Sie beantwortet: Hält sich das Programm an seine Architektur, und wo nicht?

Sie braucht Schichten. Legen Sie sie in `.codeskimr.toml` fest und scannen Sie neu ([Projekte und Scan](projekte-und-scan.md#schichten)). Ohne Schichten steht auf dem Bildschirm „Keine Schichten zu zeigen“.

## Die Karte lesen

- Ein Kasten je Schicht, eine Zeile je Schicht, äußere Schichten oben (die Reihenfolge Ihrer `[[layers]]`).
- Eine Linie zwischen zwei Kästen heißt „nutzt“. Ihre Beschriftung zählt die Verweise dahinter.
- Linien gegen die festgelegte Schichtung sind in der Verstoßfarbe markiert, und die Legende zeigt, wie viele es sind. Ein Hinweis oben listet die Verstöße („Anzeigen“ wählt sie aus). Linien, die zu einem Zyklus gehören, sind ebenfalls markiert.

Das Layout berechnet CodeSkimr selbst (geschichtetes Zeichnen), sodass dasselbe Projekt immer dasselbe Bild ergibt.

## Bedienung

- Das Mausrad zoomt; mit gehaltener `Leertaste` ziehen verschiebt; `F` (oder der Knopf „Einpassen“) passt die Karte ins Fenster ein.
- Ein Klick auf einen Kasten wählt ihn. Seine Linien sind nach Richtung gefärbt („Nutzt“ und „Genutzt von“), alle übrigen werden schwach. Das Seitenfeld zeigt die Schicht, ihre Zahl der Verweise, was sie **nutzt** und wovon sie **genutzt wird**, jeweils mit der Anzahl.
- Der Schalter „alle Verbindungen schwach“ zeigt oder verbirgt die Linien, die nicht zur Auswahl gehören; „nur Verstöße“ verbirgt jede Linie, die die Schichtung nicht verletzt.
- `Enter` bei gewählter Schicht öffnet den Explorer für das Projekt.

## Bekannte Grenzen

Die Karte zeigt nur Beziehungen zwischen Schichten. Welche Richtung zwischen zwei Schichten **erlaubt** ist, gehört noch nicht zum Projektmodell; was als „gegen die Schichtung“ gilt, folgt dem Schichtengraphen, den der Scan aufgezeichnet hat (siehe das offene Issue #45 zur Schichtregel). Nehmen Sie es als Hinweis, nicht als Urteil.
