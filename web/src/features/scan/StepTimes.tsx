import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { seconds } from "../projects/format";
import { MS_PER_SECOND, PERCENT, PERCENT_SIGN, STEP_TEXT } from "./consts";
import { slowestStep, totalMs } from "./run";
import type { StepTime } from "./types";
import styles from "./StepTimes.module.css";

function took(ms: number): string {
  return t(I18N_KEYS.SCAN_STEPS_UNIT, { seconds: seconds(ms / MS_PER_SECOND, currentLanguage()) });
}

/** The name of a step; a step that a newer server reports and this page does not know is shown by
 * its label. */
function stepName(label: string): string {
  const key = STEP_TEXT[label];
  return key === undefined ? label : t(key);
}

/** How long each step of writing the snapshot took, folded away until asked for, so that the step
 * that slows a large scan can be seen without the log. Nothing for a scan without times. */
export function StepTimes(props: { steps: readonly StepTime[] }) {
  const slowest = () => slowestStep(props.steps);
  const share = (ms: number) => Math.round((ms / Math.max(1, totalMs(props.steps))) * PERCENT);
  return (
    <Show when={props.steps.length > 0}>
      <details class={styles.details}>
        <summary class={styles.summary}>{t(I18N_KEYS.SCAN_STEPS_TITLE)}</summary>
        <ul class={styles.list}>
          <For each={props.steps}>
            {(time) => (
              <li class={styles.item} classList={{ [styles.slowest ?? ""]: time === slowest() }}>
                <span>{stepName(time.step)}</span>
                <span class={styles.track}>
                  <span
                    class={styles.fill}
                    style={{ width: share(time.duration_ms) + PERCENT_SIGN }}
                  />
                </span>
                <span class={styles.time}>{took(time.duration_ms)}</span>
              </li>
            )}
          </For>
          <li class={`${styles.item} ${styles.total}`}>
            <span>{t(I18N_KEYS.SCAN_STEPS_TOTAL)}</span>
            <span />
            <span class={styles.time}>{took(totalMs(props.steps))}</span>
          </li>
        </ul>
      </details>
    </Show>
  );
}
