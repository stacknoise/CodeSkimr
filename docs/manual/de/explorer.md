# Explorer und Suche

Der Explorer zeigt das aktive Projekt ([Projekte und Scan](projekte-und-scan.md)) aus seinem neuesten Snapshot.

## Baum

Schicht, Namensraum, Typ; unter einem Typ die Gruppen „Felder usw.“ und „Methoden“ sowie verschachtelte Typen mit ihren Membern. Typen, die zu keiner Schicht gehören, stehen zuletzt. Am Anfang ist alles zugeklappt und die Übersicht des Projekts zu sehen.

- Ein Klick auf das Dreieck klappt nur auf oder zu. Ein Klick auf einen nicht gewählten Typ wählt ihn aus; ein Klick auf den gewählten Typ klappt ihn auf oder zu. Gruppen klappen von selbst auf und zu. Eine Auswahl ändert nie, was sonst offen ist.
- Der Filter über dem Baum behält die Typen, deren Name den Text enthält, mit aufgeklappter Schicht und aufgeklapptem Namensraum.
- Tasten: `↑` `↓` bewegen, `→` klappt auf (oder geht zum ersten Kind), `←` klappt zu (oder geht zum Elternknoten), `Pos1` und `Ende` springen, `Enter` und `Leertaste` wählen oder klappen, `Strg+←` klappt alles zu, `Strg+.` zeigt die Auswahl.

Der Baum ist virtualisiert: Er bleibt auch bei Hunderttausenden Einträgen schnell.

## Details

Für den gewählten Typ: Brotkrumenpfad, Name mit Art und Sichtbarkeit, Dokumentation, zehn Metrik-Kacheln und eine Tabelle der Member (Chips Alle, Eigenschaften, Methoden, „nur öffentlich“).

Die Metriken sind unter anderem Codezeilen, zyklomatische Komplexität, Wartbarkeitsindex und Instabilität. Jede ist gegen Grenzwerte aus der Konfiguration (`metrics.*` in `config/default.toml`) als gut, mittel oder schlecht gefärbt. **Neben der Farbe steht immer die Zahl**; der Tooltip einer Spalte nennt die Grenzen.

Sprachen der Basis-Stufe ([Übersicht](index.md)) haben keinen Wartbarkeitsindex und keine Dokumentation oder Sichtbarkeit.

## Quelltext-Spalte

Die rechte Spalte zeigt Datei, Position und Code des gewählten Members (oder des Typs). Der Code wird **so, wie er jetzt ist**, aus dem Projektordner gelesen, nicht aus dem Scan: Haben Sie die Datei seitdem bearbeitet, können Zeilen bis zum nächsten Scan verschoben sein. Nur ein Projekt, das ein Ordner auf Ihrem Rechner ist, hat Quelltext; bei einem Git-Projekt sagt die Spalte das. Eine fehlende Datei ergibt `SRC-201`, eine Datei über 4 MiB `SRC-501`; eine Deklaration über 400 Zeilen wird mit Hinweis gekürzt. Pfade außerhalb des Projektordners werden abgelehnt, auch über Verknüpfungen.

Quelltext zu lesen braucht das Installations-Token. Der Browser hat es nach `codeskimr open`; Sie müssen nichts tun.

## Schnellsuche

`Strg+K` (`Cmd+K` auf dem Mac) oder das Suchfeld in der Kopfleiste öffnet die Suche über das aktive Projekt.

- Tippen, um Typen und Member nach Namen zu finden (unscharf). Ein Präfix grenzt ein: `t:` Typen, `m:` Member, `>` Befehle (zu einem Bildschirm gehen, das Projekt neu scannen). Die Chips Alle, Typen, Member und Befehle tun dasselbe.
- `↑` `↓` wählen, `Enter` öffnet den Treffer im Explorer, `Tab` und `Umschalt+Tab` wechseln den Chip, `Strg+C` kopiert den qualifizierten Namen, wenn kein Text der Eingabe markiert ist, `Esc` schließt.
- Es werden höchstens 50 Treffer gezeigt, beste zuerst. Ein Projekt ohne Scan hat keine Treffer.

## Fußzeile

Die Fußzeile jedes Bildschirms zeigt die Verbindung, die Zahl der Projekte und Typen, wie lange das Programm zum Start brauchte, seinen Speicher und den Sprachumschalter `EN | DE`. Sie zeigt nur gemessene Werte.

## Im Editor öffnen

Der Explorer kann ein Member über einen Link in Ihrem Editor öffnen. Die Vorlage ist `editor.url_template` (und die erlaubten Link-Schemata `editor.allowed_schemes`) in den Einstellungen oder der Konfiguration; der Einstellungsbildschirm hat ein Feld dafür. Diese Knöpfe sind aus, bis die Position eines Treffers verfügbar ist.
