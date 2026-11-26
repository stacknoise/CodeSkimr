# Dialog „Umbenennen prüfen“ der Explorer-Ansicht (ADR 0110).

rename-action = Umbenennen prüfen…
rename-title = Umbenennen prüfen
rename-subtitle = CodeSkimr ändert nichts. Es zeigt, was betroffen wäre.
rename-new-name = Neuer Name
rename-check = Prüfen
rename-checking = Prüfe …
rename-hint = Neuen Namen eingeben und Enter drücken.
rename-invalid-name = Das ist kein brauchbarer neuer Name. Erlaubt sind Buchstaben, Ziffern und Unterstriche, und er darf nicht der alte sein.
rename-level2 = Stufe 2: Stellen vom Indexer geprüft
rename-level1 =
    { $unchecked ->
        [one] Stufe 1: { $unchecked } Stelle ist vorläufig
       *[other] Stufe 1: { $unchecked } Stellen sind vorläufig
    }
rename-tile-places = Stellen anzupassen
rename-tile-sure = { $count } sicher
rename-tile-files = Dateien
rename-tile-layers =
    { $count ->
        [one] { $count } Schicht
       *[other] { $count } Schichten
    }
rename-tile-conflicts =
    { $count ->
        [one] Namenskonflikt
       *[other] Namenskonflikte
    }
rename-tile-texts = Text-Treffer
rename-tile-check-yourself = selbst prüfen
rename-conflict-label = Konflikt
rename-conflict-same = { $name } gibt es schon an derselben Stelle mit gleicher Signatur. Nach der Umbenennung gäbe es zwei Symbole mit diesem Namen und dieser Signatur.
rename-conflict-other = { $name } gibt es schon an derselben Stelle mit anderer Signatur. Das kann ein Überladen sein; es hängt von der Sprache ab.
rename-public-label = Öffentlich
rename-public-text = Das Symbol ist öffentlich. Aufrufer außerhalb dieses Projekts sieht CodeSkimr nicht.
rename-sites = Code-Stellen · { $total }
rename-texts = Text-Treffer, die CodeSkimr nicht zuordnen kann · { $total }
rename-more = … { $count } weitere
rename-not-searched = Der Projektordner wurde nicht nach Text durchsucht (Git-Quelle oder kein Recht, Quelltext zu lesen).
rename-skipped =
    { $count ->
        [one] { $count } Datei konnte nicht gelesen werden.
       *[other] { $count } Dateien konnten nicht gelesen werden.
    }
rename-site-sure = sicher
rename-site-probable = wahrscheinlich
rename-site-unchecked = vorläufig
rename-site-declaration = Deklaration
rename-text-documentation = Doku
rename-text-configuration = Konfiguration
rename-text-string = Zeichenkette
rename-text-nameof = nameof, sicher
rename-text-comment = Kommentar
rename-text-code = Code
rename-footer = CodeSkimr sagt nicht, ob der Code danach kompiliert. Das prüft der Compiler.
rename-copy = Liste kopieren
rename-json = Als JSON kopieren
rename-copied = Kopiert
rename-copy-failed = Nicht kopiert
rename-close = Schließen
rename-failed = Die Prüfung hat nicht geklappt. Versuche es noch einmal.
rename-esc = Esc
rename-by-file = nach Datei
