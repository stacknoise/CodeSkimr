# Dependency graph screen.

graph-title = Dependencies
graph-mode-namespaces = Namespaces
graph-mode-type-focus = Type focus
graph-depth-one = 1 level
graph-depth-two = 2 levels
graph-toggle-weak = all connections weak
graph-toggle-violations-only = violations only
graph-hint-navigation = Mouse wheel zooms · Space + drag pans
graph-action-fit = Fit to view
graph-legend-uses = uses
graph-legend-used-by = used by
graph-legend-violation = against layering ({ $count })
graph-summary = { $nodes } nodes · { $edges } edges
graph-violation-title =
    { $count ->
        [one] 1 violation in the project:
       *[other] { $count } violations in the project:
    }
graph-violation-show = Show

nav-graph = Dependencies
graph-subtitle = Layers of { $project }
graph-aria-map = Dependency map
graph-aria-selection = Selection
graph-selected-kind = Layer
graph-stat-references = References
graph-stat-uses = Uses
graph-stat-used-by = Used by
graph-uses-title = Uses · { $count }
graph-used-by-title = Used by · { $count }
graph-link-references = { $count } references
graph-violation-pair = { $from } → { $to } ({ $count })
graph-empty-title = No layers to show
graph-empty-text = This project has no layers yet. Declare them in .codeskimr.toml and scan again.
graph-no-project = Open a project first, then its layers show up here.
graph-no-edges = No dependencies between the layers were found yet.
graph-failed = The map could not be loaded.
graph-fit-key = F
graph-open-key = Enter
