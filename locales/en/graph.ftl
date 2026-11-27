# Dependency graph screen.

graph-title = Dependencies
graph-level-label = Level
graph-mode-layers = Layers
graph-mode-namespaces = Namespaces
graph-mode-type-focus = Type focus
graph-depth-one = 1 level
graph-depth-two = 2 levels
graph-toggle-weak = all connections weak
graph-toggle-violations-only = violations only
graph-hint-navigation = Mouse wheel zooms · drag or middle button pans · double click sets the focus
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
graph-empty-text = This project has no layers yet. Set them, and the map builds up with the next scan.
graph-no-project = Open a project first, then its layers show up here.
graph-no-edges = No dependencies between the layers were found yet.
graph-no-namespaces = No namespaces with types were found in this project.
graph-single-layer = A dependency map needs at least two layers. The "Namespaces" view shows the namespaces of this layer.
graph-failed = The map could not be loaded.
graph-fit-key = F
graph-open-key = Enter
graph-depth-label = Depth
graph-depth-all = all
graph-depth-1 = 1 level
graph-depth-2 = 2 levels
graph-selected-type = Type
graph-types-more = { $count } more types are not shown.
graph-types-pick = Choose a type in the explorer first.
graph-namespaces-more = { $count } more namespaces are not shown.
graph-namespace-alone = This namespace has no dependencies on others, and no other uses it.
graph-selected-namespace = Namespace
graph-toggle-critical = Highlight critical
graph-toggle-critical-only = only critical and their neighbours
graph-legend-critical = System critical
graph-critical-none = No critical symbol on this map.
graph-mode-modules = Modules
graph-selected-module = Module
graph-modules-more = { $count } more modules are not shown here; cycles and risks still count all modules.
graph-modules-table-title = Modules
graph-modules-col-module = Module
graph-modules-col-in = in
graph-modules-col-out = out
graph-modules-col-stability = Stability
graph-modules-col-types = Types
graph-modules-badge-cycle = in a cycle
graph-modules-badge-violations = { $count ->
    [one] 1 violation
   *[other] { $count } violations
}
graph-modules-col-in-hint = How many places in other modules use this module.
graph-modules-col-out-hint = How many places in this module use other modules.
graph-modules-col-stability-hint = Stability = out ÷ (in + out). 0 means many depend on it and it depends on little, so it changes better rarely. 1 means the opposite.
graph-modules-cycles-title = Cycles
graph-modules-cycles-none = No cycles between modules.
graph-modules-cycle-edge = { $from } → { $to }: { $count ->
    [one] 1 place
   *[other] { $count } places
}
graph-modules-cycle-resolve = Ending the cycle takes away the weakest edge: { $from } → { $to } ({ $count ->
    [one] 1 place
   *[other] { $count } places
}).
graph-modules-violations-title = Against the layering
graph-modules-violation = { $from } ({ $fromLayer }) → { $to } ({ $toLayer }): { $count ->
    [one] 1 place
   *[other] { $count } places
}
graph-modules-risks-title = Stands out
graph-modules-risk = { $module } (stability { $stability }) is used in { $incoming ->
    [one] 1 place
   *[other] { $incoming } places
} and itself depends on { $other } (stability { $otherStability }), which changes more often.
graph-modules-places-show = { $count ->
    [one] Show the place
   *[other] Show the { $count } places
}
graph-modules-places-hide = Hide the places
graph-modules-place-uses = { $from } → { $to }
graph-modules-places-more = { $count ->
    [one] 1 more place is not shown.
   *[other] { $count } more places are not shown.
}
graph-modules-places-failed = The places could not be loaded.
graph-modules-legend-cycle = in a cycle
graph-toggle-violations-only-hint = Only available in the "Layers" view: that is where layering violations are drawn as connections. For modules the side panel lists them under "Against the layering".
graph-modules-badge-cycle-hint = Cycle: this module depends on itself through other modules (A needs B, B needs A). The side panel names the weakest edge that ends the cycle.
