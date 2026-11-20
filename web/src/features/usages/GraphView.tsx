import { createEffect, createMemo, createSignal, For, on, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { onEnter } from "../../shared/keys/keydown";
import { cx } from "../../shared/ui/cx";
import { KindBadge } from "../explorer/KindBadge";
import { fit, transformOf, zoomAt } from "../graph/viewport";
import { useNavigation } from "../graph/useNavigation";
import type { Size, View } from "../graph/types";
import {
  COLUMN,
  GRAPH,
  MISSING,
  ROLE,
  START_MIN_SCALE,
  ZOOM_BUTTON_NOTCHES,
  ZOOM_SYMBOL,
} from "./consts";
import { buildGraph, type Card, type Column, type Edge, type Graph } from "./graph";
import { highlightOf, type Highlight } from "./highlight";
import { Legend } from "./Legend";
import type { UsagesAnswer } from "./types";
import styles from "./GraphView.module.css";

function heading(column: Column): string {
  const level = column.level;
  if (column.kind === COLUMN.focus) {
    return t(I18N_KEYS.USAGES_FOCUS);
  }
  if (column.kind === COLUMN.callers) {
    return t(I18N_KEYS.USAGES_CALLERS_LEVEL, { level });
  }
  return column.level === 1
    ? t(I18N_KEYS.USAGES_CALLEES)
    : t(I18N_KEYS.USAGES_CALLEES_LEVEL, { level });
}

/** What the layer field of a card says on hover: the layer, or that the symbol has none. */
function layerTitle(layer: string): string {
  return layer === MISSING
    ? t(I18N_KEYS.USAGES_LAYER_NONE_TITLE)
    : t(I18N_KEYS.USAGES_LAYER_TITLE, { layer });
}

function cardStyle(card: Card) {
  return {
    left: `${card.x}px`,
    top: `${card.y}px`,
    width: `${GRAPH.columnWidth}px`,
    height: `${card.height}px`,
  };
}

/** The two lines of a card: kind, name and count, then the owner and the layer. The count and the
 * layer say what they are on hover. */
function CardText(props: { card: Card }) {
  const card = () => props.card;
  return (
    <>
      <span class={styles.line}>
        <KindBadge kind={card().place.kind} />
        <span class={styles.name}>{card().member}</span>
        <Show when={card().count > 0}>
          <span
            class={styles.count}
            title={t(I18N_KEYS.USAGES_COUNT_TITLE, { count: card().count })}
          >
            {card().count}
          </span>
        </Show>
      </span>
      <span class={styles.context}>
        <span class={styles.owner}>{card().owner}</span>
        <Show when={card().place.layer ?? MISSING}>
          {(layer) => (
            <span class={styles.layer} title={layerTitle(layer())}>
              {layer()}
            </span>
          )}
        </Show>
      </span>
    </>
  );
}

function CardButton(props: {
  card: Card;
  picked: boolean;
  dim: boolean;
  onPick: (card: Card) => void;
  onOpen: (card: Card) => void;
}) {
  const card = () => props.card;
  return (
    <button
      type="button"
      class={cx(
        styles.card,
        card().focus && styles.focus,
        card().test && styles.test,
        props.picked && styles.picked,
        props.dim && styles.dim,
      )}
      style={cardStyle(card())}
      title={card().place.qualified_name}
      onClick={() => props.onPick(card())}
      onDblClick={() => props.onOpen(card())}
      onKeyDown={(event) => onEnter(event, () => props.onOpen(card()))}
    >
      <CardText card={card()} />
    </button>
  );
}

/** Where a picture of `content` pixels, scaled, starts along one axis of an area of `length`
 * pixels: in the middle if it fits, else with `focus` (a point of the picture) in the middle,
 * without leaving a gap at either end. */
function startAlong(length: number, content: number, scale: number, focus: number): number {
  const shown = content * scale;
  if (shown <= length) {
    return (length - shown) / GRAPH.half;
  }
  return Math.min(0, Math.max(length - shown, length / GRAPH.half - focus * scale));
}

/** The first view of a graph: all of it in the middle of the area, as large as fits but not
 * smaller than `START_MIN_SCALE`. A graph too large for that is centred on the focus. */
export function startView(graph: Graph, area: Size): View {
  const whole = fit(graph, area);
  const scale = Math.max(whole.scale, START_MIN_SCALE);
  if (scale === whole.scale) {
    return whole;
  }
  const focus = graph.cards.find((card) => card.focus);
  const x = focus ? focus.x + GRAPH.columnWidth / GRAPH.half : 0;
  const y = focus ? focus.y + focus.height / GRAPH.half : 0;
  return {
    scale,
    x: startAlong(area.width, graph.width, scale, x),
    y: startAlong(area.height, graph.height, scale, y),
  };
}

function ZoomButtons(props: { onZoom: (notches: number) => void; onFit: () => void }) {
  return (
    <div class={styles.zoom}>
      <button
        type="button"
        title={t(I18N_KEYS.USAGES_ZOOM_IN)}
        aria-label={t(I18N_KEYS.USAGES_ZOOM_IN)}
        onClick={() => props.onZoom(-ZOOM_BUTTON_NOTCHES)}
      >
        {ZOOM_SYMBOL.in}
      </button>
      <button
        type="button"
        title={t(I18N_KEYS.USAGES_ZOOM_OUT)}
        aria-label={t(I18N_KEYS.USAGES_ZOOM_OUT)}
        onClick={() => props.onZoom(ZOOM_BUTTON_NOTCHES)}
      >
        {ZOOM_SYMBOL.out}
      </button>
      <button
        type="button"
        title={t(I18N_KEYS.USAGES_ZOOM_FIT)}
        aria-label={t(I18N_KEYS.USAGES_ZOOM_FIT)}
        onClick={props.onFit}
      >
        {ZOOM_SYMBOL.fit}
      </button>
    </div>
  );
}

/** The view of the graph and what moves it: the wheel zooms around the pointer, a drag on the
 * empty ground or with the middle button pans, the buttons zoom and fit. A new graph starts in the middle of the area. */
function useView(graph: () => Graph) {
  let area: HTMLDivElement | undefined;
  const [view, setView] = createSignal<View>({ scale: 1, x: 0, y: 0 });
  const sizeOf = (): Size => ({
    width: area?.clientWidth ?? 0,
    height: area?.clientHeight ?? 0,
  });
  createEffect(on(graph, (current) => setView(startView(current, sizeOf()))));
  const navigation = useNavigation(() => area, view, setView);
  const centre = () => ({ x: sizeOf().width / GRAPH.half, y: sizeOf().height / GRAPH.half });
  return {
    view,
    navigation,
    attach: (element: HTMLDivElement) => (area = element),
    zoom: (steps: number) => setView(zoomAt(view(), steps, centre())),
    fit: () => setView(fit(graph(), sizeOf())),
  };
}

function Headings(props: { columns: Column[] }) {
  return (
    <For each={props.columns}>
      {(column) => (
        <span class={styles.heading} style={{ left: `${column.x}px` }}>
          {heading(column)}
          <Show when={column.hidden > 0}>
            <b class={styles.more}>{t(I18N_KEYS.USAGES_CARDS_MORE, { count: column.hidden })}</b>
          </Show>
        </span>
      )}
    </For>
  );
}

/** The lines between the cards. After a click, the lines of the picked card keep their colour and
 * the others turn faint, like the weak edges of the maps of screen 3. */
function Edges(props: { graph: Graph; highlight: Highlight }) {
  const dimmed = (edge: Edge) => props.highlight.active && !props.highlight.lit(edge);
  return (
    <svg class={styles.edges} aria-hidden={true}>
      <For each={props.graph.edges}>
        {(edge) => (
          <path
            d={edge.d}
            class={cx(styles.edge, styles[edge.tone], dimmed(edge) && styles.dimEdge)}
          />
        )}
      </For>
    </svg>
  );
}

/** The plane that is moved and scaled: column headings, edges and cards. */
function Plane(props: {
  graph: Graph;
  view: View;
  picked: number | null;
  highlight: Highlight;
  onPick: (card: Card) => void;
  onOpen: (card: Card) => void;
}) {
  return (
    <div
      class={styles.plane}
      style={{
        width: `${props.graph.width}px`,
        height: `${props.graph.height}px`,
        transform: transformOf(props.view),
      }}
    >
      <Headings columns={props.graph.columns} />
      <Edges graph={props.graph} highlight={props.highlight} />
      <For each={props.graph.cards}>
        {(card) => (
          <CardButton
            card={card}
            picked={card.place.id === props.picked}
            dim={props.highlight.active && !props.highlight.related(card.place.id)}
            onPick={props.onPick}
            onOpen={props.onOpen}
          />
        )}
      </For>
    </div>
  );
}

/** The graph of screen 6: callers on the left, the symbol in focus, what it uses on the right
 * (mock 6). A click on a card shows that symbol in the side panel, a double click makes it the focus. The picture can be zoomed and moved
 * like the map of screen 3. */
export function GraphView(props: {
  answer: UsagesAnswer;
  picked: number | null;
  onPick: (card: Card) => void;
  onOpen: (card: Card) => void;
}) {
  const graph = createMemo(() => buildGraph(props.answer));
  const state = useView(graph);
  const highlight = createMemo(() => highlightOf(graph(), props.picked));
  return (
    <div class={styles.frame}>
      <div
        ref={state.attach}
        class={cx(styles.canvas, state.navigation.grabbing() && styles.grab)}
        role={ROLE.region}
        aria-label={t(I18N_KEYS.USAGES_GRAPH_LABEL)}
        title={t(I18N_KEYS.USAGES_HINT_NAVIGATION)}
        onPointerDown={state.navigation.start}
        onPointerMove={state.navigation.move}
        onPointerUp={state.navigation.stop}
        onPointerCancel={state.navigation.stop}
      >
        <Plane
          graph={graph()}
          view={state.view()}
          picked={props.picked}
          highlight={highlight()}
          onPick={props.onPick}
          onOpen={props.onOpen}
        />
        <ZoomButtons onZoom={state.zoom} onFit={state.fit} />
      </div>
      <Legend />
    </div>
  );
}
