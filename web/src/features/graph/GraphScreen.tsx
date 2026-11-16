import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { LayersHint } from "../layers/LayersHint";
import { currentType } from "../explorer/explorer";
import { ORIENTATION, SIDE } from "../../shared/layout/consts";
import { Splitter } from "../../shared/layout/Splitter";
import { LEVEL } from "./consts";
import { panelWidth } from "./layout";
import {
  graphData,
  graphFailed,
  graphDepth,
  graphFocus,
  graphLevel,
  setGraphDepth,
  setGraphLevel,
} from "./graph";
import { GraphCanvas } from "./GraphCanvas";
import { Legend } from "./Legend";
import { neighboursOf, violations } from "./neighbours";
import { SidePanel } from "./SidePanel";
import { Toolbar } from "./Toolbar";
import type { GraphData } from "./types";
import {
  type MapState,
  useGraphKeys,
  useGraphLoading,
  useMapState,
  useSelectionActions,
} from "./useGraphScreen";
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
      onDepth={setGraphDepth}
      weak={state.weak()}
      onWeak={state.setWeak}
      violationsOnly={state.filtering()}
      onViolationsOnly={state.setViolationsOnly}
      onFit={state.fitView}
    />
  );
}

/** The drawn map: toolbar, canvas, notes and legend. */
function MapPane(props: { state: MapState; actions: Actions; current: GraphData }) {
  const { state } = props;
  return (
    <div class={styles.map}>
      <MapToolbar state={state} actions={props.actions} />
      <GraphCanvas
        data={props.current}
        selected={state.selected()}
        onSelect={state.setSelected}
        onActivate={props.actions.focusBox}
        view={state.view()}
        onView={state.setView}
        onSize={state.setSize}
        violationsOnly={state.filtering()}
        weak={state.weak()}
      />
      <Notes current={props.current} />
      <Legend
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
      onFocus={props.actions.focusableHere() ? props.actions.focusHere : undefined}
      focusLabel={t(
        graphLevel() === LEVEL.namespaces
          ? I18N_KEYS.GRAPH_FOCUS_HERE_NAMESPACE
          : I18N_KEYS.GRAPH_FOCUS_HERE,
      )}
    />
  );
}

/** What is drawn once a project is chosen and the graph did not fail. */
function Loaded(props: { state: MapState; actions: Actions }) {
  const { state } = props;
  return (
    <Show
      when={state.data()}
      fallback={graphData() ? <EmptyMap state={state} actions={props.actions} /> : null}
    >
      {(current) => (
        <>
          <MapPane state={state} actions={props.actions} current={current()} />
          <Show when={state.selected()}>
            {(layer) => (
              <>
                <Splitter pane={panelWidth} orientation={ORIENTATION.vertical} side={SIDE.after} />
                <PanelPane
                  state={state}
                  actions={props.actions}
                  current={current()}
                  layer={layer()}
                />
              </>
            )}
          </Show>
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
