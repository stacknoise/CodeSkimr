import { createMemo, For, Show } from "solid-js";
import { currentLanguage, t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths } from "../../shared/table/ResizableHead";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { countText } from "./ChangeFilter";
import {
  CROSSING_TEXT,
  DIRECTION_TONE,
  METRIC_TEXT,
  METRICS_COLUMNS,
  METRICS_LIST,
  METRICS_TABLE_KEY,
} from "./consts";
import { deltaText, metricRows, metricValue, narrowedTo, type MetricRow } from "./metrics";
import { METRICS_ORDER, metricsValue, sortedBy } from "./order";
import { SortHeads } from "./SortHeads";
import type { Listed, MetricChangeOut } from "./types";
import parts from "./Parts.module.css";
import styles from "./MetricsTab.module.css";

/** The two lists of the view, by their id in `METRICS_LIST`. */
export type MetricsSide = keyof typeof METRICS_ORDER;

const columns = {
  [METRICS_LIST.worse]: createTableColumns(
    METRICS_TABLE_KEY + METRICS_LIST.worse,
    METRICS_COLUMNS.map((column) => column.id),
  ),
  [METRICS_LIST.better]: createTableColumns(
    METRICS_TABLE_KEY + METRICS_LIST.better,
    METRICS_COLUMNS.map((column) => column.id),
  ),
};

/** The text of a word of the answer, or the word itself if the screen does not know it. */
function wordOf(table: Record<string, MessageKey | undefined>, word: string): string {
  const key = table[word];
  return key === undefined ? word : t(key);
}

/** The cell "Change": the size of the move and, where it crossed the bound of the rating, a note. */
function Change(props: { row: MetricRow }) {
  return (
    <>
      <Pill tone={DIRECTION_TONE[props.row.move.direction] ?? TONE.neutral}>
        {deltaText(props.row.move)}
      </Pill>
      <Show when={props.row.move.crossing}>
        {(crossing) => <span class={styles.cross}>{wordOf(CROSSING_TEXT, crossing())}</span>}
      </Show>
    </>
  );
}

/** One row: the object with the tag of its level, its file, the metric, from → to and the change. */
function MetricsRow(props: { row: MetricRow; tag: string }) {
  return (
    <tr>
      <td class={styles.object} title={props.row.item.name}>
        <span class={styles.tag} aria-hidden={true}>
          {props.tag}
        </span>
        <span class={parts.name}>{props.row.item.name}</span>
      </td>
      <td class={styles.where} title={props.row.item.path}>
        {props.row.item.path}
      </td>
      <td>{wordOf(METRIC_TEXT, props.row.move.metric)}</td>
      <td class={parts.number}>
        {metricValue(props.row.move.from)} → {metricValue(props.row.move.to)}
      </td>
      <td class={parts.number}>
        <Change row={props.row} />
      </td>
    </tr>
  );
}

/** The table of one list: sortable by every head, with a handle for the width of every column. */
function MetricsGrid(props: { side: MetricsSide; rows: readonly MetricRow[]; tag: string }) {
  const [order, sortBy] = METRICS_ORDER[props.side];
  const grid = columns[props.side];
  const sorted = createMemo(() => sortedBy(props.rows, order(), metricsValue, currentLanguage()));
  return (
    <div class={styles.scroll}>
      <table
        class={cx(parts.table)}
        style={grid.style()}
        aria-label={t(I18N_KEYS.CHANGE_METRICS_LIST_LABEL)}
      >
        <ColumnWidths columns={grid} />
        <thead>
          <SortHeads
            columns={grid}
            heads={METRICS_COLUMNS}
            sort={order()}
            onSort={sortBy}
            numberClass={parts.number}
          />
        </thead>
        <tbody>
          <For each={sorted()}>{(row) => <MetricsRow row={row} tag={props.tag} />}</For>
        </tbody>
      </table>
    </div>
  );
}

/** One of the two lists: its heading with the count of the objects shown and the table. */
export function Side(props: {
  side: MetricsSide;
  title: string;
  list: Listed<MetricChangeOut>;
  metric: string;
  tag: string;
  tone: PillTone;
}) {
  const items = () => narrowedTo(props.list, props.metric);
  return (
    <section class={cx(parts.card, styles.side)} aria-label={props.title}>
      <header class={styles.sideHead}>
        <h2 class={cx(styles.sideTitle, styles[props.tone])}>{props.title}</h2>
        <Pill tone={props.tone}>{items().length}</Pill>
        <span class={styles.order}>
          {countText({
            shown: items().length,
            listed: props.list.items.length,
            total: props.list.total,
          })}
        </span>
      </header>
      <MetricsGrid side={props.side} rows={metricRows(items())} tag={props.tag} />
    </section>
  );
}
