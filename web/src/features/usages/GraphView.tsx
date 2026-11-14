import { createMemo, For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { KindBadge } from "../explorer/KindBadge";
import { COLUMN, GRAPH, MISSING, ROLE } from "./consts";
import { buildGraph, type Card, type Column } from "./graph";
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

function CardButton(props: { card: Card; onPick: (card: Card) => void }) {
  const card = () => props.card;
  return (
    <button
      type="button"
      class={cx(styles.card, card().focus && styles.focus, card().test && styles.test)}
      style={{
        left: `${card().x}px`,
        top: `${card().y}px`,
        width: `${GRAPH.columnWidth}px`,
        height: `${card().height}px`,
      }}
      title={card().place.qualified_name}
      onClick={() => props.onPick(card())}
    >
      <span class={styles.line}>
        <KindBadge kind={card().place.kind} />
        <span class={styles.name}>{card().name}</span>
        <Show when={card().count > 0}>
          <span class={styles.count}>{card().count}</span>
        </Show>
      </span>
      <span class={styles.context}>{card().place.layer ?? MISSING}</span>
    </button>
  );
}

/** The graph of screen 6: callers on the left, the symbol in focus, what it uses on the right
 * (mock 6). A click on a card makes that symbol the focus. */
export function GraphView(props: { answer: UsagesAnswer; onPick: (card: Card) => void }) {
  const graph = createMemo(() => buildGraph(props.answer));
  return (
    <div class={styles.frame}>
      <div class={styles.canvas} role={ROLE.region} aria-label={t(I18N_KEYS.USAGES_GRAPH_LABEL)}>
        <div
          class={styles.plane}
          style={{ width: `${graph().width}px`, height: `${graph().height}px` }}
        >
          <For each={graph().columns}>
            {(column) => (
              <span class={styles.heading} style={{ left: `${column.x}px` }}>
                {heading(column)}
                <Show when={column.hidden > 0}>
                  <b class={styles.more}>
                    {t(I18N_KEYS.USAGES_CARDS_MORE, { count: column.hidden })}
                  </b>
                </Show>
              </span>
            )}
          </For>
          <svg class={styles.edges} aria-hidden={true}>
            <For each={graph().edges}>
              {(edge) => <path d={edge.d} class={cx(styles.edge, styles[edge.tone])} />}
            </For>
          </svg>
          <For each={graph().cards}>
            {(card) => <CardButton card={card} onPick={props.onPick} />}
          </For>
        </div>
      </div>
      <Legend />
    </div>
  );
}
