import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, HeadRow } from "../../shared/table/ResizableHead";
import { KindBadge } from "../explorer/KindBadge";
import {
  CAUSE,
  CAUSE_MOVED,
  MARK,
  ROLE,
  ROW_ID_PREFIX,
  SCROLL_NEAREST,
  SYMBOL_COLUMNS,
  SYMBOL_TABLE_KEY,
  VISIBILITY_TEXT,
} from "./consts";
import { changeRowKey, selectChangeRow } from "./change";
import { nextRow, pillsOf } from "./rows";
import type { ImpactRow } from "./types";
import parts from "./Parts.module.css";
import styles from "./SymbolTable.module.css";

const columns = createTableColumns(
  SYMBOL_TABLE_KEY,
  SYMBOL_COLUMNS.map((column) => column.id),
);

/** The colour of the mark in front of a symbol, by what happened to it. */
const MARK_STYLE: Record<string, string | undefined> = {
  [CAUSE.added]: parts.markAdded,
  [CAUSE.changed]: parts.markChanged,
  [CAUSE.removed]: parts.markRemoved,
  [CAUSE_MOVED]: parts.markChanged,
};

/** The text of the visibility of a symbol, or a dash if the language has none. */
function visibilityText(visibility: string | null): string {
  const key = VISIBILITY_TEXT[visibility ?? ""];
  return key === undefined ? t(I18N_KEYS.SCAN_UNAVAILABLE) : t(key);
}

/** The pill of the column "State": how many places are open, or that all were adapted. */
function State(props: { row: ImpactRow }) {
  return (
    <Show when={props.row.impact} fallback={<Pill>{t(I18N_KEYS.CHANGE_VIEW_STATE_NONE)}</Pill>}>
      {(impact) => (
        <Pill tone={impact().not_touched > 0 ? TONE.bad : TONE.good}>
          {impact().not_touched > 0
            ? t(I18N_KEYS.CHANGE_VIEW_STATE_OPEN, { count: impact().not_touched })
            : t(I18N_KEYS.CHANGE_VIEW_STATE_ADAPTED)}
        </Pill>
      )}
    </Show>
  );
}

/** One row of the table; a click selects it. */
function SymbolRow(props: { row: ImpactRow }) {
  const selected = () => changeRowKey() === props.row.key;
  return (
    <tr
      id={`${ROW_ID_PREFIX}${props.row.key}`}
      aria-selected={selected()}
      class={selected() ? styles.selected : undefined}
      onClick={() => selectChangeRow(props.row.key)}
    >
      <td>
        <span class={cx(parts.mark, MARK_STYLE[props.row.cause])}>{MARK[props.row.cause]}</span>
        <KindBadge kind={props.row.symbol.kind} />{" "}
        <span class={cx(parts.name, props.row.cause === CAUSE.removed && parts.gone)}>
          {props.row.symbol.qualified_name}
        </span>
      </td>
      <td>
        <span class={styles.pills}>
          <For each={pillsOf(props.row)}>
            {(pill) => <Pill tone={pill.tone}>{t(pill.label)}</Pill>}
          </For>
        </span>
      </td>
      <td class={styles.visibility}>{visibilityText(props.row.symbol.visibility)}</td>
      <td class={parts.number}>
        {props.row.impact?.usages.total ?? t(I18N_KEYS.SCAN_UNAVAILABLE)}
      </td>
      <td class={parts.number}>
        <State row={props.row} />
      </td>
    </tr>
  );
}

/** Moves the selection with the arrow keys, Home and End, and keeps the selected row in view. */
function moveSelection(event: KeyboardEvent, rows: readonly ImpactRow[]): void {
  const index = rows.findIndex((row) => row.key === changeRowKey());
  const next = nextRow(event.key, index, rows.length);
  const row = next === null ? undefined : rows[next];
  if (row !== undefined) {
    selectChangeRow(row.key);
    document.getElementById(`${ROW_ID_PREFIX}${row.key}`)?.scrollIntoView?.(SCROLL_NEAREST);
    event.preventDefault();
  }
}

/** The table of the symbols the difference removes, changes or adds, with the places that are
 * still open. It is one grid: the arrow keys move the selection (blueprint 15.4). */
export function SymbolTable(props: { rows: readonly ImpactRow[] }) {
  return (
    <div class={parts.wrap}>
      <table
        class={cx(parts.table, styles.table)}
        style={columns.style()}
        role={ROLE.grid}
        tabindex={0}
        aria-label={t(I18N_KEYS.CHANGE_VIEW_TABLE_LABEL)}
        aria-activedescendant={
          changeRowKey() === null ? undefined : `${ROW_ID_PREFIX}${changeRowKey()}`
        }
        onKeyDown={(event) => moveSelection(event, props.rows)}
      >
        <ColumnWidths columns={columns} />
        <thead>
          <HeadRow columns={columns} heads={SYMBOL_COLUMNS} numberClass={parts.number} />
        </thead>
        <tbody>
          <For each={props.rows}>{(row) => <SymbolRow row={row} />}</For>
        </tbody>
      </table>
    </div>
  );
}
