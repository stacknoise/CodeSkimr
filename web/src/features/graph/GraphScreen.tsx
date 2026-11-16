import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { LayersHint } from "../layers/LayersHint";
import { currentType } from "../explorer/explorer";
import { LEVEL } from "./consts";
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

function Message(props: { title?: string; text: string; hint?: boolean }) {
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

/** Whether the map shows one layer only, which has nothing to depend on. */
function singleLayer(current: GraphData): boolean {
  return graphLevel() === LEVEL.layers && current.boxes.length === 1;
}

/** The notes under the map: types left out, no edges. */
function Notes(props: { current: GraphData }) {
  return (
    <>
      <Show when={moreTypes(props.current)}>
        {(count) => <p class={styles.note}>{t(I18N_KEYS.GRAPH_TYPES_MORE, { count: count() })}</p>}
      </Show>
      <Show when={props.current.links.length === 0}>
        <p class={styles.note}>
          {t(singleLayer(props.current) ? I18N_KEYS.GRAPH_SINGLE_LAYER : I18N_KEYS.GRAPH_NO_EDGES)}
        </p>
      </Show>
    </>
  );
}

/** The drawn map: toolbar, canvas, notes and legend. */
function MapPane(props: { state: MapState; actions: Actions; current: GraphData }) {
  const { state } = props;
  return (
    <div class={styles.map}>
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
      <GraphCanvas
        data={props.current}
        selected={state.selected()}
        onSelect={state.setSelected}
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
      kind={
        graphLevel() === LEVEL.types
          ? t(I18N_KEYS.GRAPH_SELECTED_TYPE)
          : t(I18N_KEYS.GRAPH_SELECTED_KIND)
      }
      onFocus={props.actions.focusableHere() ? props.actions.focusHere : undefined}
    />
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
          <Message
            title={t(I18N_KEYS.GRAPH_EMPTY_TITLE)}
            text={t(I18N_KEYS.GRAPH_EMPTY_TEXT)}
            hint
          />
        ) : null
      }
    >
      {(current) => (
        <>
          <MapPane state={state} actions={props.actions} current={current()} />
          <Show when={state.selected()}>
            {(layer) => (
              <PanelPane
                state={state}
                actions={props.actions}
                current={current()}
                layer={layer()}
              />
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
