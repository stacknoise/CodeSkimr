import { batch, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { annotationIndex, annotationsAvailable } from "../annotations/store";
import {
  highlightCritical,
  onlyCriticalAndNeighbours,
  setHighlightCritical,
  setOnlyCriticalAndNeighbours,
} from "../annotations/flags";
import { LayersHint } from "../layers/LayersHint";
import { currentType } from "../explorer/explorer";
import { ORIENTATION, SIDE } from "../../shared/layout/consts";
import { Splitter } from "../../shared/layout/Splitter";
import { LEVEL } from "./consts";
import { modulesPanelWidth, panelWidth } from "./layout";
import {
  graphData,
  graphFailed,
  graphDepth,
  graphFocus,
  graphLevel,
  modulesInfo,
  setGraphDepth,
  setGraphNamespace,
  setGraphLevel,
} from "./graph";
import { hasCriticalBox, isCriticalBox, narrowToCritical } from "./critical";
import { GraphCanvas } from "./GraphCanvas";
import { Legend } from "./Legend";
import { ModulesPanel } from "./ModulesPanel";
import { neighboursOf, violations } from "./neighbours";
import { SidePanel } from "./SidePanel";
import { Toolbar, type CriticalSwitches } from "./Toolbar";
import type { GraphData, LayerBox } from "./types";
import {
  type MapState,
  useGraphKeys,
  useGraphLoading,
  useMapState,
  useSelectionActions,
} from "./useGraphScreen";
import { SlowLoad } from "../../shared/ui/SlowLoad";
import styles from "./GraphScreen.module.css";

function Message(props: { title?: string | undefined; text: string; hint?: boolean }) {
  return (
    <div class={styles.empty}>
      <Show when={props.title}>
        <h2>{props.title}</h2>
      </Show>
      <p>{props.text}</p>
      <Show when={props.hint}>
        <LayersHint />
      </Show>
    </div>
  );
}

type Actions = ReturnType<typeof useSelectionActions>;

/** What the selected box is, by level. */
const SELECTED_KIND = {
  layers: I18N_KEYS.GRAPH_SELECTED_KIND,
  namespaces: I18N_KEYS.GRAPH_SELECTED_NAMESPACE,
  types: I18N_KEYS.GRAPH_SELECTED_TYPE,
  modules: I18N_KEYS.GRAPH_SELECTED_MODULE,
} as const;

/** Whether the map shows one layer only, which has nothing to depend on. */
function singleLayer(current: GraphData): boolean {
  return graphLevel() === LEVEL.layers && current.boxes.length === 1;
}

/** The note for a map without a line: by level, and a single layer needs a second one. */
function noEdgesText(current: GraphData) {
  if (graphLevel() === LEVEL.namespaces) {
    return I18N_KEYS.GRAPH_NAMESPACE_ALONE;
  }
  return singleLayer(current) ? I18N_KEYS.GRAPH_SINGLE_LAYER : I18N_KEYS.GRAPH_NO_EDGES;
}

/** The notes under the map: types left out, no edges. */
function Notes(props: { current: GraphData }) {
  return (
    <>
      <Show when={moreTypes(props.current)}>
        {(count) => (
          <p class={styles.note}>
            {t(
              graphLevel() === LEVEL.namespaces
                ? I18N_KEYS.GRAPH_NAMESPACES_MORE
                : I18N_KEYS.GRAPH_TYPES_MORE,
              { count: count() },
            )}
          </p>
        )}
      </Show>
      <Show when={props.current.links.length === 0}>
        <p class={styles.note}>{t(noEdgesText(props.current))}</p>
      </Show>
    </>
  );
}

/** The switches of the critical symbols; the layer and module levels have none, a layer is not annotated, and a
 * server without annotations has none either. */
function criticalSwitches(): CriticalSwitches | null {
  if (
    annotationsAvailable() === false ||
    graphLevel() === LEVEL.layers ||
    graphLevel() === LEVEL.modules
  ) {
    return null;
  }
  return {
    highlight: highlightCritical(),
    onHighlight: setHighlightCritical,
    only: onlyCriticalAndNeighbours(),
    onOnly: setOnlyCriticalAndNeighbours,
  };
}

/** Changes the depth and keeps the selected namespace in the middle: the map names it with fewer
 * or more segments, and the server finds it again by its name. */
function chooseDepth(state: MapState, depth: number): void {
  batch(() => {
    const chosen = state.selected();
    if (graphLevel() === LEVEL.namespaces && chosen !== null) {
      setGraphNamespace(chosen);
    }
    setGraphDepth(depth);
  });
}

/** The toolbar of the map; it stays when there is nothing to draw, so that another level can be
 * chosen. */
function MapToolbar(props: { state: MapState; actions: Actions }) {
  const { state } = props;
  return (
    <Toolbar
      level={graphLevel()}
      onLevel={setGraphLevel}
      focusable={currentType() !== null || graphFocus() !== null}
      onTypeFocus={props.actions.focusType}
      depth={graphDepth()}
      onDepth={(depth) => chooseDepth(state, depth)}
      weak={state.weak()}
      onWeak={state.setWeak}
      violationsOnly={state.filtering()}
      onViolationsOnly={state.setViolationsOnly}
      critical={criticalSwitches()}
      onFit={state.fitView}
    />
  );
}

/** What the critical switches do to the drawn map: the map as it is shown (narrowed to the
 * critical boxes and their neighbours, or whole) and whether a box is drawn as critical. */
function useCriticalView(current: () => GraphData) {
  const active = () => criticalSwitches() !== null;
  return {
    shown: () =>
      active() && onlyCriticalAndNeighbours()
        ? narrowToCritical(current(), annotationIndex(), graphLevel())
        : current(),
    isCritical: (box: LayerBox) =>
      active() && highlightCritical() && isCriticalBox(annotationIndex(), box, graphLevel()),
    nothing: () =>
      active() &&
      onlyCriticalAndNeighbours() &&
      !hasCriticalBox(current(), annotationIndex(), graphLevel()),
    marked: () => active() && highlightCritical(),
  };
}

/** The drawn map: toolbar, canvas, notes and legend. */
function MapPane(props: { state: MapState; actions: Actions; current: GraphData }) {
  const { state } = props;
  const view = useCriticalView(() => props.current);
  return (
    <div class={styles.map}>
      <MapToolbar state={state} actions={props.actions} />
      <GraphCanvas
        data={view.shown()}
        critical={view.isCritical}
        selected={state.selected()}
        onSelect={state.setSelected}
        onOpen={props.actions.focusBox}
        view={state.view()}
        onView={state.setView}
        onSize={state.setSize}
        violationsOnly={state.filtering()}
        weak={state.weak()}
      />
      <Notes current={props.current} />
      <Show when={view.nothing()}>
        <p class={styles.note}>{t(I18N_KEYS.GRAPH_CRITICAL_NONE)}</p>
      </Show>
      <Legend
        critical={view.marked()}
        cycles={graphLevel() === LEVEL.modules}
        layers={props.current.boxes.length}
        links={props.current.links.length}
        violations={violations(props.current.links).length}
      />
    </div>
  );
}

/** The map with nothing on it: the toolbar stays, the text says why and what to do. The layers
 * need a layer to be set; the other levels only a scanned project. */
function EmptyMap(props: { state: MapState; actions: Actions }) {
  const layers = () => graphLevel() === LEVEL.layers;
  return (
    <div class={styles.map}>
      <MapToolbar state={props.state} actions={props.actions} />
      <Message
        title={layers() ? t(I18N_KEYS.GRAPH_EMPTY_TITLE) : undefined}
        text={t(layers() ? I18N_KEYS.GRAPH_EMPTY_TEXT : I18N_KEYS.GRAPH_NO_NAMESPACES)}
        hint={layers()}
      />
    </div>
  );
}

/** The side panel of the selected layer or type. */
function PanelPane(props: {
  state: MapState;
  actions: Actions;
  current: GraphData;
  layer: string;
}) {
  return (
    <SidePanel
      layer={props.layer}
      neighbours={neighboursOf(props.layer, props.current.links)}
      violations={violations(props.current.links)}
      onSelect={props.state.setSelected}
      onOpen={props.actions.openExplorer}
      kind={t(SELECTED_KIND[graphLevel()])}
    />
  );
}

/** The panel beside the map: the table of the level "Modules", else the details of the selected
 * box. */
function Panels(props: { state: MapState; actions: Actions; current: GraphData }) {
  const { state } = props;
  const resizer = (pane = panelWidth) => (
    <Splitter pane={pane} orientation={ORIENTATION.vertical} side={SIDE.after} />
  );
  return (
    <Show
      when={graphLevel() !== LEVEL.modules}
      fallback={
        <Show when={modulesInfo()}>
          {(info) => (
            <>
              {resizer(modulesPanelWidth)}
              <ModulesPanel
                info={info()}
                selected={state.selected()}
                onSelect={state.setSelected}
              />
            </>
          )}
        </Show>
      }
    >
      <Show when={state.selected()}>
        {(layer) => (
          <>
            {resizer()}
            <PanelPane
              state={state}
              actions={props.actions}
              current={props.current}
              layer={layer()}
            />
          </>
        )}
      </Show>
    </Show>
  );
}

/** What is drawn once a project is chosen and the graph did not fail. */
function Loaded(props: { state: MapState; actions: Actions }) {
  const { state } = props;
  return (
    <Show
      when={state.data()}
      fallback={
        graphData() ? (
          <EmptyMap state={state} actions={props.actions} />
        ) : (
          <SlowLoad waiting={() => graphData() === null} class={styles.empty} />
        )
      }
    >
      {(current) => (
        <>
          <MapPane state={state} actions={props.actions} current={current()} />
          <Panels state={state} actions={props.actions} current={current()} />
        </>
      )}
    </Show>
  );
}

/** Screen 3, the dependency map of the layers (blueprint B.3, mock `Graph.dc.html`). The layout
 * comes from the server (`graph.layers`); this screen only draws it, with zoom, pan and the
 * selection panel. */
export function GraphScreen() {
  const state = useMapState();
  const actions = useSelectionActions(state);
  useGraphLoading(state);
  useGraphKeys(state, actions.openExplorer);
  return (
    <section class={styles.screen} aria-label={t(I18N_KEYS.GRAPH_ARIA_MAP)}>
      <Show
        when={state.project() !== null}
        fallback={<Message text={t(I18N_KEYS.GRAPH_NO_PROJECT)} />}
      >
        <Show when={!graphFailed()} fallback={<Message text={t(I18N_KEYS.GRAPH_FAILED)} />}>
          <Loaded state={state} actions={actions} />
        </Show>
      </Show>
    </section>
  );
}

/** How many types the map on the type level leaves out, `null` if none. */
function moreTypes(data: GraphData): number | null {
  const more = (data.focus?.users_more ?? 0) + (data.focus?.used_more ?? 0);
  return more > 0 ? more : null;
}
