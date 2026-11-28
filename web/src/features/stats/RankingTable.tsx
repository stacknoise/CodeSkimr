import { createEffect, For, Match, Show, Switch } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { KindBadge } from "../explorer/KindBadge";
import { count } from "../projects/format";
import { ASSESSMENTS, OPEN_ARROW, RATING_POOR, TAB, REASON_TEXT, SCROLL_NEAREST } from "./consts";
import { mostUsages } from "./geometry";
import type { RankingName, Row } from "./types";
import { KeepButton } from "./KeepButton";
import { UsageBar } from "./UsageBar";
import styles from "./RankingTable.module.css";

/** The header of the value columns of each ranking. */
const COLUMNS: Record<RankingName, MessageKey[]> = {
  most_used: [I18N_KEYS.STATS_COLUMN_USAGES],
  unused: [I18N_KEYS.STATS_COLUMN_ASSESSMENT, I18N_KEYS.STATS_COLUMN_REASON],
  complexity: [I18N_KEYS.STATS_COLUMN_COMPLEXITY, I18N_KEYS.STATS_COLUMN_LINES],
  size: [I18N_KEYS.STATS_COLUMN_LINES, I18N_KEYS.STATS_COLUMN_COMPLEXITY],
  coupling: [I18N_KEYS.STATS_COLUMN_EFFERENT, I18N_KEYS.STATS_COLUMN_AFFERENT],
};

function number(value: number | null): string {
  return value === null ? t(I18N_KEYS.STATS_VALUE_MISSING) : count(value, currentLanguage());
}

/** The usages of a row as text: "28 certain, up to 73", or only the certain ones if all are. */
export function usageText(row: Row): string {
  const args = { certain: row.certain, total: row.total };
  return t(
    row.total > row.certain ? I18N_KEYS.STATS_USAGE_PAIR : I18N_KEYS.STATS_USAGE_CERTAIN,
    args,
  );
}

function reasonText(row: Row): string {
  return row.reason === null ? t(I18N_KEYS.STATS_VALUE_MISSING) : t(REASON_TEXT[row.reason]);
}

/** The pill with the assessment of a row without usages. */
export function AssessmentPill(props: { row: Row }) {
  const found = () => ASSESSMENTS.find((entry) => entry.id === props.row.assessment);
  return (
    <Show when={found()}>
      {(entry) => (
        <span title={t(entry().label)}>
          <Pill tone={entry().tone}>{t(entry().short)}</Pill>
        </span>
      )}
    </Show>
  );
}

/** The value cells of a row, which depend on the ranking. */
function Values(props: { row: Row; ranking: RankingName; most: number }) {
  const complexity = () => (
    <td class={cx(styles.number, props.row.complexity_rating === RATING_POOR && styles.poor)}>
      {number(props.row.complexity)}
    </td>
  );
  return (
    <Switch>
      <Match when={props.ranking === TAB.mostUsed}>
        <td class={styles.usage}>
          <UsageBar row={props.row} most={props.most} />
          <span>{usageText(props.row)}</span>
        </td>
      </Match>
      <Match when={props.ranking === TAB.unused}>
        <td>
          <AssessmentPill row={props.row} />
        </td>
        <td>{reasonText(props.row)}</td>
      </Match>
      <Match when={props.ranking === TAB.complexity}>
        {complexity()}
        <td class={styles.number}>{number(props.row.loc)}</td>
      </Match>
      <Match when={props.ranking === TAB.size}>
        <td class={styles.number}>{number(props.row.loc)}</td>
        {complexity()}
      </Match>
      <Match when={props.ranking === TAB.coupling}>
        <td class={styles.number}>{number(props.row.efferent)}</td>
        <td class={styles.number}>{number(props.row.afferent)}</td>
      </Match>
    </Switch>
  );
}

/** Keeps a row in view while it is selected: the arrow keys move the selection, not the page.
 * Returns the `ref` of the row. */
function useInView(selected: () => boolean): (row: HTMLTableRowElement) => void {
  let line: HTMLTableRowElement | undefined;
  createEffect(() => {
    if (selected()) {
      line?.scrollIntoView?.(SCROLL_NEAREST);
    }
  });
  return (row) => {
    line = row;
  };
}

/** The cell with the arrow that opens the symbol in the explorer. */
function OpenCell(props: { row: Row; onOpen: (row: Row) => void }) {
  return (
    <td class={styles.go}>
      <button
        type="button"
        aria-label={t(I18N_KEYS.STATS_OPEN_EXPLORER_ROW, { name: props.row.name })}
        onClick={() => props.onOpen(props.row)}
      >
        {OPEN_ARROW}
      </button>
    </td>
  );
}

function TableRow(props: {
  row: Row;
  rank: number;
  ranking: RankingName;
  most: number;
  selected: boolean;
  keep: KeepTarget;
  onSelect: (row: Row) => void;
  onOpen: (row: Row) => void;
}) {
  const line = useInView(() => props.selected);
  return (
    <tr
      ref={line}
      class={cx(props.selected && styles.selected)}
      aria-current={props.selected}
      onClick={() => props.onSelect(props.row)}
      onDblClick={() => props.onOpen(props.row)}
    >
      <td class={styles.rank}>{props.rank}</td>
      <td class={styles.name}>
        <KindBadge kind={props.row.kind} />{" "}
        <span title={props.row.qualified_name}>{props.row.name}</span>
      </td>
      <td class={styles.path} title={props.row.path}>
        {props.row.path}
      </td>
      <Values row={props.row} ranking={props.ranking} most={props.most} />
      <Show when={props.ranking === TAB.unused}>
        <KeepButton row={props.row} project={props.keep.project} editable={props.keep.editable} />
      </Show>
      <OpenCell row={props.row} onOpen={props.onOpen} />
    </tr>
  );
}

/** Where "Keep on purpose" writes to: the project, and whether it has a project file. */
export interface KeepTarget {
  project: number;
  editable: boolean;
}

/** The header row of a ranking. */
function Head(props: { ranking: RankingName }) {
  return (
    <thead>
      <tr>
        <th />
        <th>{t(I18N_KEYS.STATS_COLUMN_NAME)}</th>
        <th>{t(I18N_KEYS.STATS_COLUMN_FILE)}</th>
        <For each={COLUMNS[props.ranking]}>{(column) => <th>{t(column)}</th>}</For>
        <Show when={props.ranking === TAB.unused}>
          <th />
        </Show>
        <th />
      </tr>
    </thead>
  );
}

/** The table of a ranking. A click selects a row, a double click or its arrow opens the symbol in
 * the explorer. */
export function RankingTable(props: {
  ranking: RankingName;
  rows: Row[];
  selected: number | null;
  keep: KeepTarget;
  onSelect: (row: Row) => void;
  onOpen: (row: Row) => void;
}) {
  const most = () => mostUsages(props.rows);
  return (
    <table class={styles.table} aria-label={t(I18N_KEYS.STATS_TABLE_LABEL)}>
      <Head ranking={props.ranking} />
      <tbody>
        <For each={props.rows}>
          {(row, index) => (
            <TableRow
              row={row}
              rank={index() + 1}
              ranking={props.ranking}
              most={most()}
              selected={row.id === props.selected}
              keep={props.keep}
              onSelect={props.onSelect}
              onOpen={props.onOpen}
            />
          )}
        </For>
      </tbody>
    </table>
  );
}
