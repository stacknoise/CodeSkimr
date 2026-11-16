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
  graphNamespace,
  loadGraph,
  resetGraph,
  setGraphFocus,
  setGraphLevel,
  setGraphNamespace,
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
  // On the type and namespace levels the one in the middle is selected,
  // so the panel shows what surrounds it.
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
      [graphLevel, graphDepth, graphFocus, graphNamespace],
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

/** The type of the selected box on the type level, `null` elsewhere. */
function selectedTypeOf(state: MapState): number | null | undefined {
  const name = state.selected();
  return graphLevel() === LEVEL.types
    ? state.data()?.boxes.find((box) => box.name === name)?.symbol
    : null;
}

/** Puts a box in the middle of the map: a namespace on the namespace level, a type on the type
 * level. */
function useFocusActions(state: MapState) {
  const { selected, data } = state;
  const focusBox = (name: string) => {
    const symbol = data()?.boxes.find((box) => box.name === name)?.symbol;
    if (graphLevel() === LEVEL.namespaces) {
      setGraphNamespace(name);
    } else if (graphLevel() === LEVEL.types && symbol !== null && symbol !== undefined) {
      setGraphFocus(symbol);
    }
  };
  const focusableHere = () => {
    const name = selected();
    if (graphLevel() === LEVEL.namespaces) {
      return name !== null && name !== data()?.focus?.name;
    }
    const type = selectedTypeOf(state);
    return type !== null && type !== undefined && type !== graphFocus();
  };
  const focusHere = () => {
    const name = selected();
    if (name !== null && focusableHere()) {
      focusBox(name);
    }
  };
  const focusType = () => {
    const chosen = currentType();
    if (chosen !== null) {
      setGraphFocus(chosen);
      setGraphLevel(LEVEL.types);
    }
  };
  return { focusType, focusBox, focusHere, focusableHere };
}

/** What the screen does with the selected box: open it in the explorer, make it the focus. */
export function useSelectionActions(state: MapState) {
  const { project } = state;
  const openExplorer = () => {
    const id = project();
    if (id !== null) {
      const type = selectedTypeOf(state);
      if (type !== null && type !== undefined) {
        requestFocus({ type, member: null, kind: null });
      }
      openProject(id);
      navigate(EXPLORER_ROUTE.path);
    }
  };
  return { openExplorer, ...useFocusActions(state) };
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
