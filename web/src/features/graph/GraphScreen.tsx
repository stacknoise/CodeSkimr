import { createEffect, createMemo, createSignal, on, onCleanup, onMount, Show } from "solid-js";
import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject, openProject } from "../../shared/project/active";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { projects, refreshProjects } from "../projects/projects";
import { KEY } from "./consts";
import { graphData, graphFailed, loadGraph, resetGraph, wantedLayer, wantLayer } from "./graph";
import { GraphCanvas } from "./GraphCanvas";
import { Legend } from "./Legend";
import { neighboursOf, violations } from "./neighbours";
import { SidePanel } from "./SidePanel";
import { Toolbar } from "./Toolbar";
import type { Size, View } from "./types";
import { fit } from "./viewport";
import styles from "./GraphScreen.module.css";

function Message(props: { title?: string; text: string }) {
  return (
    <div class={styles.empty}>
      <Show when={props.title}>
        <h2>{props.title}</h2>
      </Show>
      <p>{props.text}</p>
    </div>
  );
}

/** Screen 3, the dependency map of the layers (blueprint B.3, mock `Graph.dc.html`). The layout
 * comes from the server (`graph.layers`); this screen only draws it, with zoom, pan and the
 * selection panel. */
export function GraphScreen() {
  const [selected, setSelected] = createSignal<string | null>(null);
  const [weak, setWeak] = createSignal(true);
  const [violationsOnly, setViolationsOnly] = createSignal(false);
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
  createEffect(() => {
    const layer = wantedLayer();
    const known = graphData()?.boxes.some((box) => box.name === layer);
    if (layer !== null && known) {
      setSelected(layer);
      wantLayer(null);
    }
  });
  useKeydown((event) => {
    if (event.ctrlKey || event.altKey || event.metaKey || isTyping(event.target)) {
      return;
    }
    if (event.key.toLowerCase() === KEY.fit) {
      fitView();
    } else if (event.key === KEY.open && selected() !== null && !isButton(event.target)) {
      openExplorer();
    }
  });
  const openExplorer = () => {
    const id = project();
    if (id !== null) {
      openProject(id);
      navigate(EXPLORER_ROUTE.path);
    }
  };
  const brokenLinks = () => violations(graphData()?.links ?? []);
  return (
    <section class={styles.screen} aria-label={t(I18N_KEYS.GRAPH_ARIA_MAP)}>
      <Show when={project() !== null} fallback={<Message text={t(I18N_KEYS.GRAPH_NO_PROJECT)} />}>
        <Show when={!graphFailed()} fallback={<Message text={t(I18N_KEYS.GRAPH_FAILED)} />}>
          <Show
            when={data()}
            fallback={
              graphData() ? (
                <Message
                  title={t(I18N_KEYS.GRAPH_EMPTY_TITLE)}
                  text={t(I18N_KEYS.GRAPH_EMPTY_TEXT)}
                />
              ) : null
            }
          >
            {(current) => (
              <>
                <div class={styles.map}>
                  <Toolbar
                    weak={weak()}
                    onWeak={setWeak}
                    violationsOnly={violationsOnly()}
                    onViolationsOnly={setViolationsOnly}
                    onFit={fitView}
                  />
                  <GraphCanvas
                    data={current()}
                    selected={selected()}
                    onSelect={setSelected}
                    view={view()}
                    onView={setView}
                    onSize={setSize}
                    violationsOnly={violationsOnly()}
                    weak={weak()}
                  />
                  <Show when={current().links.length === 0}>
                    <p class={styles.note}>{t(I18N_KEYS.GRAPH_NO_EDGES)}</p>
                  </Show>
                  <Legend
                    layers={current().boxes.length}
                    links={current().links.length}
                    violations={brokenLinks().length}
                  />
                </div>
                <Show when={selected()}>
                  {(layer) => (
                    <SidePanel
                      layer={layer()}
                      neighbours={neighboursOf(layer(), current().links)}
                      violations={brokenLinks()}
                      onSelect={setSelected}
                      onOpen={openExplorer}
                    />
                  )}
                </Show>
              </>
            )}
          </Show>
        </Show>
      </Show>
    </section>
  );
}

function isButton(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && target.tagName === "BUTTON";
}
