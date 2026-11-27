import { Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { changeRowKey } from "./change";
import { MARK } from "./consts";
import { Figure } from "./Figure";
import { Notes } from "./Notes";
import { affectedFiles, impactRows, symbolCounts } from "./rows";
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

/** The view "Impact" (mock erw-auswirkung): the figures, the table of the symbols with the notes
 * under it, and on the right the usages of the selected symbol. */
export function ImpactTab(props: { answer: ChangeAnswer }) {
  const rows = () => impactRows(props.answer);
  const selected = () => rows().find((row) => row.key === changeRowKey());
  return (
    <>
      <Figures answer={props.answer} />
      <Show
        when={rows().length > 0}
        fallback={<p class={parts.plain}>{t(I18N_KEYS.CHANGE_VIEW_NONE)}</p>}
      >
        <div class={styles.columns}>
          <div class={cx(parts.card, styles.list)}>
            <SymbolTable rows={rows()} />
            <Notes answer={props.answer} />
          </div>
          <Show when={selected()}>{(row) => <UsagesPanel row={row()} />}</Show>
        </div>
      </Show>
    </>
  );
}
