import { createMemo, For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths } from "../../shared/table/ResizableHead";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { COMPARE_COLUMNS, COMPARE_TABLE_KEY, REASON, TITLE_JOIN } from "./consts";
import { ChangeFilterBar, FilterEmpty } from "./ChangeFilter";
import { changeFilter, kindsOf, passes } from "./filter";
import { Figure } from "./Figure";
import { Notes } from "./Notes";
import { Selectors } from "./Selectors";
import { compareOrder, compareValue, sortCompareBy, sortedBy } from "./order";
import { SortHeads } from "./SortHeads";
import { answerTotal, compareRows, reasonCount, type CompareRow } from "./rows";
import type { ChangeAnswer } from "./types";
import parts from "./Parts.module.css";
import styles from "./CompareTab.module.css";

/** The four figures: new, changed (with the reasons), removed and moved symbols. */
function Figures(props: { answer: ChangeAnswer }) {
  return (
    <div class={parts.figures}>
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_CARD_ADDED)}
        value={String(props.answer.added.total)}
        hint={t(I18N_KEYS.CHANGE_VIEW_CARD_ADDED_HINT)}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_CARD_CHANGED)}
        value={String(props.answer.changed.total)}
        hint={t(
          props.answer.changed.is_cut
            ? I18N_KEYS.CHANGE_VIEW_CARD_CHANGED_HINT_CUT
            : I18N_KEYS.CHANGE_VIEW_CARD_CHANGED_HINT,
          {
            signature: reasonCount(props.answer, REASON.signature),
            visibility: reasonCount(props.answer, REASON.visibility),
            content: reasonCount(props.answer, REASON.content),
            shown: props.answer.changed.items.length,
          },
        )}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_CARD_REMOVED)}
        value={String(props.answer.removed.total)}
        hint={t(I18N_KEYS.CHANGE_VIEW_CARD_REMOVED_HINT)}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_CARD_MOVED)}
        value={String(props.answer.moved.total)}
        hint={t(I18N_KEYS.CHANGE_VIEW_CARD_MOVED_HINT)}
      />
    </div>
  );
}

/** The full text of a row, for the tooltip of a cell that was cut: the name and its signature. */
function titleOf(row: CompareRow): string {
  return [row.name, row.signature].filter((part) => part !== null).join(TITLE_JOIN);
}

const columns = createTableColumns(
  COMPARE_TABLE_KEY,
  COMPARE_COLUMNS.map((column) => column.id),
);

/** One row of the list: the pills, the symbol with its signature, and the file. */
function Row(props: { row: CompareRow }) {
  return (
    <tr>
      <td class={styles.pills}>
        <For each={props.row.pills}>{(pill) => <Pill tone={pill.tone}>{t(pill.label)}</Pill>}</For>
      </td>
      <td class={styles.symbol} title={titleOf(props.row)}>
        <span class={parts.name}>{props.row.name}</span>
        <Show when={props.row.signature}>
          {(signature) => <span class={parts.sig}> {signature()}</span>}
        </Show>
      </td>
      <td class={styles.where} title={props.row.where}>
        {props.row.where}
      </td>
    </tr>
  );
}

/** The table of the rows, in a box that scrolls: the heads sort it and have the handles for the widths. */
function RowsTable(props: { rows: readonly CompareRow[] }) {
  return (
    <div class={styles.scroll}>
      <table
        class={parts.table}
        style={columns.style()}
        aria-label={t(I18N_KEYS.CHANGE_VIEW_LIST_LABEL)}
      >
        <ColumnWidths columns={columns} />
        <thead>
          <SortHeads
            columns={columns}
            heads={COMPARE_COLUMNS}
            sort={compareOrder()}
            onSort={sortCompareBy}
          />
        </thead>
        <tbody>
          <For each={props.rows}>{(row) => <Row row={row} />}</For>
        </tbody>
      </table>
    </div>
  );
}

/** The filter row and the rows that pass it, in a box that scrolls on its own. */
function SymbolList(props: { answer: ChangeAnswer }) {
  const rows = createMemo(() => compareRows(props.answer));
  const visible = createMemo(() =>
    sortedBy(
      rows().filter((row) => passes(row.facts, changeFilter(), false)),
      compareOrder(),
      compareValue,
      currentLanguage(),
    ),
  );
  return (
    <div class={cx(parts.card, styles.list)}>
      <ChangeFilterBar
        kinds={kindsOf(rows().map((row) => row.facts))}
        withOpen={false}
        count={{
          shown: visible().length,
          listed: rows().length,
          total: answerTotal(props.answer),
        }}
      />
      <Show when={visible().length > 0} fallback={<FilterEmpty />}>
        <RowsTable rows={visible()} />
      </Show>
      <Notes answer={props.answer} />
    </div>
  );
}

/** The view "Comparison" (mock 29): the two scans, the figures and the list of the symbols. The
 * list scrolls in its own box, so the page keeps its height. */
export function CompareTab(props: { project: number; answer: ChangeAnswer | null }) {
  return (
    <>
      <Selectors project={props.project} />
      <Show when={props.answer}>
        {(answer) => (
          <>
            <Figures answer={answer()} />
            <Show
              when={compareRows(answer()).length > 0}
              fallback={
                <>
                  <p class={parts.plain}>
                    {t(
                      answer().base.snapshot === answer().target.snapshot
                        ? I18N_KEYS.CHANGE_VIEW_SAME
                        : I18N_KEYS.CHANGE_VIEW_NONE,
                    )}
                  </p>
                  <Notes answer={answer()} />
                </>
              }
            >
              <SymbolList answer={answer()} />
            </Show>
            <p class={parts.plain}>{t(I18N_KEYS.CHANGE_VIEW_COMPARE_NOTE)}</p>
          </>
        )}
      </Show>
    </>
  );
}
