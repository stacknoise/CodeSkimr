# Abhängigkeitsgraph.

graph-title = Abhängigkeiten
graph-level-label = Ebene
graph-mode-layers = Schichten
graph-mode-namespaces = Namespaces
graph-mode-type-focus = Typ-Fokus
graph-depth-one = 1 Stufe
graph-depth-two = 2 Stufen
graph-toggle-weak = alle Verbindungen schwach
graph-toggle-violations-only = nur Verstöße
graph-hint-navigation = Mausrad zoomt · Leertaste + Ziehen verschiebt
graph-action-fit = Einpassen
graph-legend-uses = nutzt
graph-legend-used-by = wird genutzt
graph-legend-violation = gegen Schichtung ({ $count })
graph-summary = { $nodes } Knoten · { $edges } Kanten
graph-violation-title =
    { $count ->
        [one] 1 Verstoß im Projekt:
       *[other] { $count } Verstöße im Projekt:
    }
graph-violation-show = Ansehen

nav-graph = Abhängigkeiten
graph-aria-map = Abhängigkeitskarte
graph-aria-selection = Auswahl
graph-selected-kind = Schicht
graph-stat-references = Verweise
graph-stat-uses = Nutzt
graph-stat-used-by = Genutzt von
graph-uses-title = Nutzt · { $count }
graph-used-by-title = Wird genutzt von · { $count }
graph-link-references = { $count } Verweise
graph-violation-pair = { $from } → { $to } ({ $count })
graph-empty-title = Keine Schichten zu zeigen
graph-empty-text = Dieses Projekt hat noch keine Schichten. Lege sie in .codeskimr.toml an und scanne neu.
graph-no-project = Öffne zuerst ein Projekt, dann erscheinen hier seine Schichten.
graph-no-edges = Zwischen den Schichten wurden noch keine Abhängigkeiten gefunden.
graph-failed = Die Karte konnte nicht geladen werden.
graph-fit-key = F
graph-open-key = Enter
graph-depth-label = Tiefe
graph-depth-all = alle
graph-depth-1 = 1 Stufe
graph-depth-2 = 2 Stufen
