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
