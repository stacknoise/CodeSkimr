import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { BUDGET_ROWS } from "./consts";
import type { BudgetTargets } from "./types";
import styles from "./Budgets.module.css";

/** Measured values in milliseconds by budget key; a budget without one was not measured here. */
export type Measured = Partial<Record<keyof BudgetTargets, number>>;

function Value(props: { target: number; measured: number | undefined; rated: boolean }) {
  return (
    <Show
      when={props.measured !== undefined}
      fallback={<span class={styles.value}>{t(I18N_KEYS.SCAN_UNAVAILABLE)}</span>}
    >
      <span
        class={cx(
          styles.value,
          props.rated && ((props.measured ?? 0) <= props.target ? styles.good : styles.bad),
        )}
      >
        {t(I18N_KEYS.SCAN_BUDGET_MEASURED, { ms: Math.round(props.measured ?? 0) })}
      </span>
    </Show>
  );
}

/** The time budgets of blueprint 16.1 with their targets from the configuration and, where this
 * session measured one, the measured value (mock 5, right column). */
export function Budgets(props: {
  targets: BudgetTargets | null;
  measured: Measured;
  /** Budgets whose measured value is shown without a good or bad colour, because it was not
   * measured under the conditions of the target (for example a scan of a smaller project). */
  unrated?: ReadonlyArray<keyof BudgetTargets>;
}) {
  return (
    <div class={styles.card}>
      <div class={styles.title}>{t(I18N_KEYS.SCAN_BUDGETS_TITLE)}</div>
      <div class={styles.hint}>{t(I18N_KEYS.SCAN_BUDGETS_HINT)}</div>
      <Show when={props.targets}>
        {(targets) => (
          <For each={BUDGET_ROWS}>
            {(row) => (
              <div class={styles.row}>
                <span class={styles.name}>{t(row.name)}</span>
                <span class={styles.goal}>
                  {t(I18N_KEYS.SCAN_BUDGET_TARGET, { ms: targets()[row.key] })}
                </span>
                <Value
                  target={targets()[row.key]}
                  measured={props.measured[row.key]}
                  rated={!props.unrated?.includes(row.key)}
                />
              </div>
            )}
          </For>
        )}
      </Show>
    </div>
  );
}
