import { Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { count } from "../projects/format";
import { isApproximate } from "../usages/approximation";
import { KEY_TEXT, RATING_POOR } from "./consts";
import type { Row } from "./types";
import styles from "./SidePanel.module.css";

function Stat(props: { value: number | null; label: MessageKey; poor?: boolean }) {
  return (
    <Show when={props.value !== null}>
      <div class={styles.stat}>
        <div class={cx(styles.number, props.poor && styles.poor)}>
          {count(props.value ?? 0, currentLanguage())}
        </div>
        <div class={styles.label}>{t(props.label)}</div>
      </div>
    </Show>
  );
}

/** Why the usages of the row are not exact: an upper bound, or a language with approximate
 * usages. */
function Notes(props: { row: Row }) {
  const probable = () => props.row.total - props.row.certain;
  return (
    <>
      <Show when={probable() > 0}>
        <p class={styles.note}>
          <strong>{t(I18N_KEYS.STATS_UPPER_BOUND_TITLE)}</strong>{" "}
          {t(I18N_KEYS.STATS_UPPER_BOUND, { probable: probable(), total: props.row.total })}
        </p>
      </Show>
      <Show when={isApproximate(props.row.language)}>
        <p class={styles.note}>{t(I18N_KEYS.STATS_APPROX_NOTE)}</p>
      </Show>
    </>
  );
}

/** The details of the selected row and the two ways out of the screen: the explorer and the
 * usages. */
export function SidePanel(props: {
  row: Row | null;
  onOpen: (row: Row) => void;
  onUsages: (row: Row) => void;
}) {
  return (
    <aside class={styles.panel}>
      <Show when={props.row} fallback={<p class={styles.hint}>{t(I18N_KEYS.STATS_SELECT_HINT)}</p>}>
        {(row) => (
          <>
            <div>
              <div class={styles.kind}>{row().kind}</div>
              <h2 class={styles.title}>{row().name}</h2>
              <div class={styles.path}>{row().path}</div>
            </div>
            <div class={styles.stats}>
              <Stat value={row().certain} label={I18N_KEYS.STATS_DETAIL_CERTAIN} />
              <Stat value={row().total - row().certain} label={I18N_KEYS.STATS_DETAIL_PROBABLE} />
              <Stat
                value={row().complexity}
                label={I18N_KEYS.STATS_DETAIL_COMPLEXITY}
                poor={row().complexity_rating === RATING_POOR}
              />
              <Stat value={row().loc} label={I18N_KEYS.STATS_DETAIL_LINES} />
              <Stat value={row().efferent} label={I18N_KEYS.STATS_DETAIL_EFFERENT} />
              <Stat value={row().afferent} label={I18N_KEYS.STATS_DETAIL_AFFERENT} />
            </div>
            <Notes row={row()} />
            <button type="button" class={styles.open} onClick={() => props.onOpen(row())}>
              {t(I18N_KEYS.STATS_OPEN_EXPLORER)} <kbd>{KEY_TEXT.open}</kbd>
            </button>
            <button type="button" class={styles.usages} onClick={() => props.onUsages(row())}>
              {t(I18N_KEYS.STATS_OPEN_USAGES)} <kbd>{KEY_TEXT.usages}</kbd>
            </button>
          </>
        )}
      </Show>
    </aside>
  );
}
