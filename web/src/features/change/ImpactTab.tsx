import { createEffect, createMemo, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { changeRowKey, selectChangeRow } from "./change";
import { ChangeFilterBar, FilterEmpty } from "./ChangeFilter";
import { MARK } from "./consts";
import { Figure } from "./Figure";
import { Notes } from "./Notes";
import { changeFilter, kindsOf, passes } from "./filter";
import {
  affectedFiles,
  answerTotal,
  factsOf,
  firstToSelect,
  impactRows,
  symbolCounts,
} from "./rows";
import { SymbolTable } from "./SymbolTable";
import { UsagesPanel } from "./UsagesPanel";
import type { ChangeAnswer } from "./types";
import parts from "./Parts.module.css";
import styles from "./ImpactTab.module.css";

/** The five figures over the table: the symbols, the public ones, the usages they hit, the places
 * not adapted and the files that hold them. */
function Figures(props: { answer: ChangeAnswer }) {
  const summary = () => props.answer.summary;
  const counts = () => symbolCounts(props.answer);
  const usages = () => (summary().usages_certain ?? 0) + (summary().usages_probable ?? 0);
  return (
    <div class={parts.figures}>
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_TILE_SYMBOLS)}
        value={String(counts().added + counts().changed + counts().removed)}
        small={[
          `${MARK["added"]}${counts().added}`,
          `${MARK["changed"]}${counts().changed}`,
          `${MARK["removed"]}${counts().removed}`,
        ].join(" ")}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_TILE_PUBLIC)}
        value={String(summary().public_symbols)}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_TILE_USAGES)}
        value={String(usages())}
        small={t(I18N_KEYS.CHANGE_VIEW_TILE_USAGES_CERTAIN, {
          count: summary().usages_certain ?? 0,
        })}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_TILE_OPEN)}
        value={String(summary().usages_not_touched ?? 0)}
      />
      <Figure
        label={t(I18N_KEYS.CHANGE_VIEW_TILE_FILES)}
        value={String(affectedFiles(props.answer))}
      />
    </div>
  );
}

/** The rows of the table that pass the filter. If the selected row is held back, the selection
 * moves to the first row that is left (or to none), so the panel never shows a hidden symbol. */
function visibleRows(answer: () => ChangeAnswer) {
  const rows = createMemo(() => impactRows(answer()));
  const visible = createMemo(() =>
    rows().filter((row) => passes(factsOf(row), changeFilter(), true)),
  );
  createEffect(() => {
    const key = changeRowKey();
    if (!visible().some((row) => row.key === key)) {
      selectChangeRow(firstToSelect(visible())?.key ?? null);
    }
  });
  return { rows, visible };
}

/** The view "Impact" (mock erw-auswirkung): the figures, the table of the symbols with the filter
 * over it and the notes under it, and on the right the usages of the selected symbol. The list and
 * the panel each scroll on their own; the page keeps its height. */
export function ImpactTab(props: { answer: ChangeAnswer }) {
  const { rows, visible } = visibleRows(() => props.answer);
  const selected = () => visible().find((row) => row.key === changeRowKey());
  return (
    <>
      <Figures answer={props.answer} />
      <Show
        when={rows().length > 0}
        fallback={
          <p class={parts.plain}>
            {t(
              props.answer.base.snapshot === props.answer.target.snapshot
                ? I18N_KEYS.CHANGE_VIEW_SAME
                : I18N_KEYS.CHANGE_VIEW_NONE,
            )}
          </p>
        }
      >
        <div class={styles.columns}>
          <div class={cx(parts.card, styles.list)}>
            <ChangeFilterBar
              kinds={kindsOf(rows().map(factsOf))}
              withOpen
              count={{
                shown: visible().length,
                listed: rows().length,
                total: answerTotal(props.answer),
              }}
            />
            <Show when={visible().length > 0} fallback={<FilterEmpty />}>
              <SymbolTable rows={visible()} />
            </Show>
            <Notes answer={props.answer} />
          </div>
          <Show when={selected()}>{(row) => <UsagesPanel row={row()} />}</Show>
        </div>
      </Show>
    </>
  );
}
