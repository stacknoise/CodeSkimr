# Regeln

Die Seite **Regeln** (linke Leiste) prüft die Architekturregeln Ihrer Projektdatei `.codeskimr.toml` gegen den letzten Scan und listet ihre Verstöße auf. Die Schichten haben eine eigene Seite **Schichten** in der linken Leiste (der Editor der Schichten, siehe [Projekte und Scan](projekte-und-scan.md)).

## Was eine Regel ist

| Regel | Bedeutung | In der Datei |
|---|---|---|
| Schichten | eine Schicht darf nur die Schichten in ihrem `may_use` nutzen | `[[layers]]` mit `may_use` |
| Keine Zyklen | Module dürfen sich nicht im Kreis voneinander abhängen | `[rules]` `cycles = "deny"` |
| Verbot | Modul A darf Modul B nicht nutzen | `[[rules.forbid]]` mit `from`, `to`, optional `reason` |
| Öffentliche Schnittstelle | das Innere eines Moduls darf nur von innen genutzt werden | `[[rules.public_api]]` mit `module`, `internal` |

Ein Verstoß ist ein Symbol, das ein Symbol erreicht, das es nicht erreichen darf, mit allen Stellen. Beide Namen sind Knöpfe, die das Symbol im Explorer zeigen; ein Zyklus hat den Knopf **Im Explorer zeigen**. Bei einem Namen, den der letzte Scan nicht kennt, ist der Knopf ausgegraut, mit dem Hinweis, neu zu scannen. **Stand festhalten** hält alle jetzigen Verstöße als Altlast fest; die CI (`codeskimr rules`) schlägt dann nur bei neuen fehl.

## Regeln im UI anlegen und ändern

Sie müssen die Datei nicht von Hand schreiben (ADR 0112). **Regel hinzufügen** öffnet ein Menü:

- **Verbot** und **Nur über die öffentliche Schnittstelle** öffnen einen Dialog. Die Modulfelder bieten die Namensräume des letzten Scans an; ein eigenes Präfix können Sie auch tippen. Unter dem Formular zeigt der Dialog, wie viele Verstöße der letzte Scan mit der Regel, wie getippt, fände (und wie viele davon schon festgehalten sind), und den Text, der in die Datei geschrieben wird. **Speichern** schreibt die Regel sofort.
- **Zyklen verbieten** ist ein Schalter; im Menü ist er aus, sobald er an ist.
- **Schichten** öffnet eine Tabelle „wer darf wen nutzen“: Die Zeile nutzt die Spalten. Ohne Haken bei „Limit“ darf eine Schicht alles nutzen; mit Limit nur die angehakten Schichten. Namen und Namensräume der Schichten bleiben im Reiter **Schichten**.

Jede Regelkarte hat einen Stift (ändern) und einen Papierkorb (löschen; vorher fragt ein Dialog). Nur die Tabellen `rules` (und das `may_use` der Schichten) werden neu geschrieben; alles andere in der Datei bleibt, ebenso Kommentare außerhalb der Regel-Tabellen. Eine ungültige Regel (ein Modul im anderen, dasselbe Paar doppelt, eine unbekannte Schicht) wird mit `CFG-101` abgelehnt, die Datei bleibt wie sie war.

Ein Git-Projekt hat keinen Projektordner, darum sind **Regel hinzufügen**, Stift und Papierkorb dort aus; tragen Sie die Regeln in die Datei im Repository ein.

## Die Code-Regeln auf der Seite „Konventionen“ prüfen

Die Seite „Konventionen“ (eigener Punkt in der linken Leiste, neben „Regeln“) prüft den Code des Projektordners gegen die Code-Regeln, so wie der Konsolenbefehl `codeskimr check` es tut (ADR 0116): eine zu lange oder zu verschachtelte Funktion, zu viele Parameter, Literale im Code und zwei Funktionen, die Kopien voneinander sind (`docs/conventions.md`). Er liest die Dateien so, wie sie jetzt sind, und sieht darum auch Änderungen, die noch kein Scan gesehen hat. Es wird nichts gespeichert.

**Jetzt prüfen** startet die Prüfung für den ganzen Ordner des Projekts. Ein großes Projekt braucht etwa so lange wie die Analyse eines Scans. Der Streifen zeigt, wie viele Dateien geprüft wurden, wie viele Funde es gibt und wie viele Dateien sich nicht analysieren ließen. Die Funde stehen in einer Tabelle, die du nach jeder Spalte sortieren und deren Spalten du verbreitern kannst; **Öffnen** öffnet die Datei an der Zeile im Editor aus den Einstellungen; **Explorer** zeigt die Funktion im Explorer, wie der letzte Scan sie hat (ausgegraut, mit Hinweis, bei einer Funktion, die dieser Scan nicht kennt, etwa in einer später geschriebenen Datei). **Als Markdown kopieren** legt das Ergebnis in die Zwischenablage, für ein Issue oder einen Kommentar. Unter der Tabelle steht, welche Regeln diese Prüfung nicht abdeckt (ein sauberes Ergebnis sagt darüber nichts) und welche Dateien sie nicht lesen konnte.

Ein Projekt von einer Git-Adresse hat keinen Ordner auf diesem Rechner; der Knopf ist dort ausgegraut.
