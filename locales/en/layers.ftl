# Setting layers: a choice from the detected namespaces (ADR 0065).

layers-title = Layers
layers-subtitle = Build layers from the detected namespaces. The choice is saved in .codeskimr.toml in the project folder.
layers-no-project = Pick a project first, the one whose layers you want to set.
layers-not-scanned = The project has not been scanned yet, so there are no namespaces to choose from. Scan it first.
layers-not-editable = A Git project has no project folder the layers could be written to. Add them to the .codeskimr.toml of the repository.
layers-load-failed = The layers of the project could not be read.
layers-namespaces-title = Detected namespaces
layers-namespaces-note = A tick selects the namespace together with everything below it.
layers-namespaces-empty = The last scan found no namespaces.
layers-types = { $count ->
    [one] { NUMBER($count) } type
   *[other] { NUMBER($count) } types
}
layers-owned-by = in "{ $layer }"
layers-expand = Expand { $name }
layers-collapse = Collapse { $name }
layers-select = Select { $name }
layers-selected-count = { $count ->
    [one] { NUMBER($count) } selected
   *[other] { NUMBER($count) } selected
}
layers-layers-title = Your layers
layers-order-note = The order counts: if a name fits several layers, the first one wins. Outer layers come first.
layers-empty = No layers yet. Select namespaces on the left and make a layer from them.
layers-new-name = Name of the new layer
layers-new-action = Create layer from selection
layers-add-selection = Add selection
layers-name-label = Name of the layer
layers-move-up = Move { $name } up
layers-move-down = Move { $name } down
layers-remove = Remove { $name }
layers-remove-namespace = Take { $name } out of the layer
layers-save = Save and scan again
layers-saving = Saving and scanning …
layers-reset = Discard changes
layers-invalid = Every layer needs a name of its own and at least one namespace.
layers-saved = Saved. The new scan is done.
layers-file-note = Saving only replaces the [[layers]] blocks of the file; the "may_use" permissions stay for layers of the same name. Comments inside the old blocks are lost.
layers-hint-title = No layers set yet
layers-hint-text = Without layers everything stays under "(no layer)" and the dependency map is empty.
layers-hint-action = Set layers
settings-layers-title = Layers
settings-layers-note = Decide which namespaces belong to which layer. The explorer and the dependency map group by them.
settings-layers-action = Edit layers
