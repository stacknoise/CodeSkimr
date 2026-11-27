import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Pill } from "../../shared/ui/Pill";
import { CAUSE, CONFIDENCE, LINE_SEPARATOR, REASON, SIGN, STATUS } from "./consts";
import { changeOnlyOpen, showOnlyOpen } from "./change";
import { diffLines, groupUsages, type DiffLine } from "./rows";
import type { ImpactRow, SymbolImpactOut, UsageOut } from "./types";
import parts from "./Parts.module.css";
import styles from "./UsagesPanel.module.css";

/** The heading over the lines of a change: what kind of change the selected symbol has. */
function headingOf(row: ImpactRow) {
  if (row.cause === CAUSE.removed) {
    return I18N_KEYS.CHANGE_VIEW_DETAIL_REMOVED;
  }
  if (row.cause === CAUSE.added) {
    return I18N_KEYS.CHANGE_VIEW_DETAIL_ADDED;
  }
  if (row.reasons.includes(REASON.signature)) {
    return I18N_KEYS.CHANGE_VIEW_DETAIL_SIGNATURE;
  }
  return row.reasons.includes(REASON.visibility)
    ? I18N_KEYS.CHANGE_VIEW_DETAIL_VISIBILITY
    : I18N_KEYS.CHANGE_VIEW_DETAIL_CONTENT;
}

/** The old and the new signature as lines with a sign, in the colours of removed and new. */
function Lines(props: { lines: DiffLine[] }) {
  return (
    <div class={styles.code}>
      <For each={props.lines}>
        {(line) => (
          <div class={line.sign === SIGN.before ? styles.before : styles.after}>
            {line.sign} {line.text}
          </div>
        )}
      </For>
    </div>
  );
}

/** One usage: its line, the symbol that holds it and how sure the index is, and whether the
 * change adapted it. */
function Usage(props: { usage: UsageOut }) {
  const adapted = () => props.usage.status === STATUS.adapted;
  return (
    <div class={styles.usage}>
      <span class={styles.line}>{props.usage.line}</span>
      <span class={styles.holder} title={`${props.usage.path}${LINE_SEPARATOR}${props.usage.line}`}>
        {props.usage.from.qualified_name}
      </span>
      <Pill tone={props.usage.confidence === CONFIDENCE.certain ? TONE.neutral : TONE.warn}>
        {t(
          props.usage.confidence === CONFIDENCE.certain
            ? I18N_KEYS.CHANGE_VIEW_CONFIDENCE_CERTAIN
            : I18N_KEYS.CHANGE_VIEW_CONFIDENCE_PROBABLE,
        )}
      </Pill>
      <Pill tone={adapted() ? TONE.good : TONE.bad}>
        {t(
          adapted()
            ? I18N_KEYS.CHANGE_VIEW_STATUS_ADAPTED
            : I18N_KEYS.CHANGE_VIEW_STATUS_NOT_TOUCHED,
        )}
      </Pill>
    </div>
  );
}

/** The usages of one file, under a head that says how many are still open. */
function Group(props: { path: string; open: number; usages: UsageOut[] }) {
  return (
    <div role="group" aria-label={props.path}>
      <div class={styles.groupHead}>
        {props.path}
        <Pill tone={props.open > 0 ? TONE.bad : TONE.good}>
          {props.open > 0
            ? t(I18N_KEYS.CHANGE_VIEW_GROUP_OPEN, { count: props.open })
            : t(I18N_KEYS.CHANGE_VIEW_GROUP_ADAPTED)}
        </Pill>
      </div>
      <For each={props.usages}>{(usage) => <Usage usage={usage} />}</For>
    </div>
  );
}

/** The two buttons that switch the list between the places still open and all. */
function Filter(props: { open: number; all: number }) {
  return (
    <div class={styles.filter} role="group" aria-label={t(I18N_KEYS.CHANGE_VIEW_FILTER_LABEL)}>
      <button
        type="button"
        class={cx(styles.toggle, changeOnlyOpen() && styles.on)}
        aria-pressed={changeOnlyOpen()}
        onClick={() => showOnlyOpen(true)}
      >
        {t(I18N_KEYS.CHANGE_VIEW_FILTER_OPEN, { count: props.open })}
      </button>
      <button
        type="button"
        class={cx(styles.toggle, !changeOnlyOpen() && styles.on)}
        aria-pressed={!changeOnlyOpen()}
        onClick={() => showOnlyOpen(false)}
      >
        {t(I18N_KEYS.CHANGE_VIEW_FILTER_ALL, { count: props.all })}
      </button>
    </div>
  );
}

/** The count of the usages, how sure they are, the filter and the usages grouped by file. */
function Usages(props: { impact: SymbolImpactOut }) {
  const items = () => props.impact.usages.items;
  const certain = () => items().filter((usage) => usage.confidence === CONFIDENCE.certain).length;
  const groups = () => groupUsages(items(), changeOnlyOpen());
  return (
    <>
      <div class={styles.summary}>
        <b>{t(I18N_KEYS.CHANGE_VIEW_USAGES_COUNT, { count: props.impact.usages.total })}</b>
        <Pill>{t(I18N_KEYS.CHANGE_VIEW_USAGES_CERTAIN, { count: certain() })}</Pill>
        <Pill tone={TONE.warn}>
          {t(I18N_KEYS.CHANGE_VIEW_USAGES_PROBABLE, { count: items().length - certain() })}
        </Pill>
        <span class={parts.spacer} />
        <Filter open={props.impact.not_touched} all={items().length} />
      </div>
      <For each={groups()}>{(group) => <Group {...group} />}</For>
      <Show when={groups().length === 0}>
        <p class={styles.hint}>{t(I18N_KEYS.CHANGE_VIEW_USAGES_ALL_ADAPTED)}</p>
      </Show>
      <Show when={props.impact.usages.is_cut}>
        <p class={styles.cut}>
          {t(I18N_KEYS.CHANGE_VIEW_USAGES_CUT, {
            shown: items().length,
            total: props.impact.usages.total,
          })}
        </p>
      </Show>
    </>
  );
}

/** The panel on the right of the impact: what changed in the selected symbol and the usages the
 * change can break, with the checklist of the places that were not touched (mock erw-auswirkung). */
export function UsagesPanel(props: { row: ImpactRow }) {
  return (
    <section class={parts.card} aria-label={t(I18N_KEYS.CHANGE_VIEW_DETAIL_LABEL)}>
      <div class={styles.top}>
        <span class={parts.figureLabel}>{t(headingOf(props.row))}</span>
        <Lines lines={diffLines(props.row)} />
        <Show when={props.row.narrowed}>
          <p class={styles.hint}>{t(I18N_KEYS.CHANGE_VIEW_DETAIL_NARROWED)}</p>
        </Show>
      </div>
      <Show
        when={props.row.impact}
        fallback={<p class={styles.hint}>{t(I18N_KEYS.CHANGE_VIEW_DETAIL_NO_CHECKLIST)}</p>}
      >
        {(impact) => <Usages impact={impact()} />}
      </Show>
      <div class={parts.notes}>
        <p class={styles.footer}>{t(I18N_KEYS.CHANGE_VIEW_CHECKLIST_NOTE)}</p>
        <p class={styles.footer}>{t(I18N_KEYS.COMPARE_DIRECT_ONLY_NOTE)}</p>
      </div>
    </section>
  );
}
