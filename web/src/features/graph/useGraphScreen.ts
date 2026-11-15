import { createEffect, createMemo, createSignal, on, onCleanup, onMount } from "solid-js";
import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { activeProject, openProject } from "../../shared/project/active";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { currentType } from "../explorer/explorer";
import { requestFocus } from "../../shared/navigation/navigation";
import { projects, refreshProjects } from "../projects/projects";
import { KEY, LEVEL } from "./consts";
import {
  graphData,
  graphDepth,
  graphFocus,
  graphLevel,
  loadGraph,
  resetGraph,
  setGraphFocus,
  setGraphLevel,
  wantedLayer,
  wantLayer,
} from "./graph";
import type { Size, View } from "./types";
import { fit } from "./viewport";

/** The state of the screen: selection, filters, view and the drawn data. */
export function useMapState() {
  const [selected, setSelected] = createSignal<string | null>(null);
  const [weak, setWeak] = createSignal(true);
  const [violationsOnly, setViolationsOnly] = createSignal(false);
  // No violation is derived on the namespace level, so the filter only acts on the layer level.
  const filtering = () => violationsOnly() && graphLevel() === LEVEL.layers;
  const [view, setView] = createSignal<View>({ scale: 1, x: 0, y: 0 });
  const [size, setSize] = createSignal<Size>({ width: 0, height: 0 });
  const project = createMemo(() => activeProject() ?? projects()[0]?.id ?? null);
  const data = () => (graphData() && graphData()!.boxes.length > 0 ? graphData() : null);
  const fitView = () => {
    const current = data();
    if (current) {
      setView(fit(current, size()));
    }
  };
  return {
    selected,
    setSelected,
    weak,
    setWeak,
    setViolationsOnly,
    filtering,
    view,
    setView,
    size,
    setSize,
    project,
    data,
    fitView,
  };
}

/** The state of the screen as the hooks and components share it. */
export type MapState = ReturnType<typeof useMapState>;

/** Loads the graph for the project and keeps the view and the selection in step with it. */
export function useGraphLoading(state: MapState): void {
  const { project, setSelected, size, fitView } = state;
  onMount(() => void refreshProjects());
  onCleanup(() => wantLayer(null));
  createEffect(
    on(project, (id) => {
      resetGraph();
      setSelected(null);
      if (id !== null) {
        void loadGraph(id);
      }
    }),
  );
  createEffect(on([() => graphData()?.snapshot, size], fitView));
  // On the type level the type in the middle is selected, so the panel shows what surrounds it.
  createEffect(on(graphData, (loaded) => loaded?.focus && setSelected(loaded.focus.name)));
  createEffect(() => {
    const layer = wantedLayer();
    const known = graphData()?.boxes.some((box) => box.name === layer);
    if (layer !== null && known) {
      setSelected(layer);
      wantLayer(null);
    }
  });
  createEffect(
    on(
      [graphLevel, graphDepth, graphFocus],
      () => {
        const id = project();
        if (id !== null) {
          setSelected(null);
          void loadGraph(id);
        }
      },
      { defer: true },
    ),
  );
}

function isButton(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && target.tagName === "BUTTON";
}

/** What the screen does with the selected box: open it in the explorer, make it the focus. */
export function useSelectionActions(state: MapState) {
  const { selected, project, data } = state;
  const selectedType = () => {
    const name = selected();
    return graphLevel() === LEVEL.types
      ? data()?.boxes.find((box) => box.name === name)?.symbol
      : null;
  };
  const openExplorer = () => {
    const id = project();
    if (id !== null) {
      const type = selectedType();
      if (type !== null && type !== undefined) {
        requestFocus({ type, member: null, kind: null });
      }
      openProject(id);
      navigate(EXPLORER_ROUTE.path);
    }
  };
  const focusType = () => {
    const chosen = currentType();
    if (chosen !== null) {
      setGraphFocus(chosen);
      setGraphLevel(LEVEL.types);
    }
  };
  const focusableHere = () => {
    const type = selectedType();
    return type !== null && type !== undefined && type !== graphFocus();
  };
  const focusHere = () => {
    const type = selectedType();
    if (type !== null && type !== undefined && type !== graphFocus()) {
      setGraphFocus(type);
    }
  };
  return { openExplorer, focusType, focusHere, focusableHere };
}

/** The keys of the screen: fit the view and open the selection. */
export function useGraphKeys(state: MapState, openExplorer: () => void): void {
  useKeydown((event) => {
    if (event.ctrlKey || event.altKey || event.metaKey || isTyping(event.target)) {
      return;
    }
    if (event.key.toLowerCase() === KEY.fit) {
      state.fitView();
    } else if (event.key === KEY.open && state.selected() !== null && !isButton(event.target)) {
      openExplorer();
    }
  });
}
