import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { Pill } from "../../shared/ui/Pill";
import { changePicked, changeReading, changeScans, loadChange, pickScans } from "./change";
import { NOT_FOUND, BASE_FIELD_ID, REASON, SCAN_ARROW, TARGET_FIELD_ID } from "./consts";
import { Figure } from "./Figure";
import { Notes } from "./Notes";
import { compareRows, reasonCount, type CompareRow } from "./rows";
import { optionText } from "./scans";
import type { ChangeAnswer } from "./types";
import parts from "./Parts.module.css";
import styles from "./CompareTab.module.css";

/** The choice of one scan: a label and a list of the scans that can be that end of the comparison. */
function ScanField(props: {
  id: string;
  label: string;
  value: number | null;
  /** Indexes into the list of the scans, newest first, that the field offers. */
  indexes: readonly number[];
  onPick: (snapshot: number) => void;
}) {
  return (
    <div class={styles.field}>
      <label class={parts.figureLabel} for={props.id}>
        {props.label}
      </label>
      <select
        id={props.id}
        class={styles.select}
        disabled={changeReading()}
        onChange={(event) => props.onPick(Number(event.currentTarget.value))}
      >
        <For each={props.indexes}>
          {(index) => {
            const scan = () => changeScans()?.scans[index];
            return (
              <Show when={scan()}>
                {(entry) => (
                  <option value={entry().snapshot} selected={entry().snapshot === props.value}>
                    {optionText(entry(), index === 0)}
                  </option>
                )}
              </Show>
            );
          }}
        </For>
      </select>
    </div>
  );
}

/** The indexes of the scans that can be the base (older than the target) and the target (newer
 * than the base). */
function offers(): { base: number[]; target: number[] } {
  const scans = changeScans()?.scans ?? [];
  const at = (snapshot: number | null, fallback: number) => {
    const found = scans.findIndex((scan) => scan.snapshot === snapshot);
    return found === NOT_FOUND ? fallback : found;
  };
  const target = at(changePicked().target, 0);
  const base = at(changePicked().base, scans.length - 1);
  const all = scans.map((_, index) => index);
  return {
    base: all.filter((index) => index > target),
    target: all.filter((index) => index < base),
  };
}

/** The two selectors of the scans and the button that compares them (mock 29). */
function Selectors(props: { project: number }) {
  return (
    <div class={styles.selectors}>
      <ScanField
        id={BASE_FIELD_ID}
        label={t(I18N_KEYS.CHANGE_VIEW_BASE)}
        value={changePicked().base}
        indexes={offers().base}
        onPick={(base) => pickScans({ ...changePicked(), base })}
      />
      <span class={styles.arrow} aria-hidden={true}>
        {SCAN_ARROW}
      </span>
      <ScanField
        id={TARGET_FIELD_ID}
        label={t(I18N_KEYS.CHANGE_VIEW_TARGET)}
        value={changePicked().target}
        indexes={offers().target}
        onPick={(target) => pickScans({ ...changePicked(), target })}
      />
      <button
        type="button"
        class={styles.run}
        disabled={changeReading()}
        onClick={() => void loadChange(props.project, changePicked())}
      >
        {t(I18N_KEYS.CHANGE_VIEW_RUN)}
      </button>
    </div>
  );
}

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
        hint={t(I18N_KEYS.CHANGE_VIEW_CARD_CHANGED_HINT, {
          signature: reasonCount(props.answer, REASON.signature),
          visibility: reasonCount(props.answer, REASON.visibility),
          content: reasonCount(props.answer, REASON.content),
        })}
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

/** One row of the list: the pills, the symbol with its signature, and the file. */
function Row(props: { row: CompareRow }) {
  return (
    <tr>
      <td class={styles.pills}>
        <For each={props.row.pills}>{(pill) => <Pill tone={pill.tone}>{t(pill.label)}</Pill>}</For>
      </td>
      <td>
        <span class={parts.name}>{props.row.name}</span>
        <Show when={props.row.signature}>
          {(signature) => <span class={parts.sig}> {signature()}</span>}
        </Show>
      </td>
      <td class={styles.where}>{props.row.where}</td>
    </tr>
  );
}

/** The view "Comparison" (mock 29): the two scans, the figures and the list of the symbols. */
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
              fallback={<p class={parts.plain}>{t(I18N_KEYS.CHANGE_VIEW_NONE)}</p>}
            >
              <div class={parts.card}>
                <table class={parts.table} aria-label={t(I18N_KEYS.CHANGE_VIEW_LIST_LABEL)}>
                  <tbody>
                    <For each={compareRows(answer())}>{(row) => <Row row={row} />}</For>
                  </tbody>
                </table>
              </div>
            </Show>
            <Notes answer={answer()} />
            <p class={parts.plain}>{t(I18N_KEYS.CHANGE_VIEW_COMPARE_NOTE)}</p>
          </>
        )}
      </Show>
    </>
  );
}
