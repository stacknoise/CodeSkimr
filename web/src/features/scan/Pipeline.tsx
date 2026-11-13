import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, seconds } from "../projects/format";
import {
  FILE_STAGE,
  PERCENT,
  PERCENT_SIGN,
  STAGE_MARKS,
  STAGE_STATE,
  STAGE_TEXT,
  STAGES,
  STATE_ATTRIBUTE,
} from "./consts";
import { lastEventAt } from "./events";
import { stageSeconds, stageState, type StageState } from "./run";
import type { Run, Stage } from "./types";
import styles from "./Pipeline.module.css";

function detail(run: Run | undefined, stage: Stage): string {
  const text = t(STAGE_TEXT[stage].detail);
  const report = run?.stages[stage];
  if (stage !== FILE_STAGE || report === undefined || report.total === 0) {
    return text;
  }
  const language = currentLanguage();
  const done = count(report.done, language);
  return t(I18N_KEYS.SCAN_STAGE_COUNTS, { text, done, total: count(report.total, language) });
}

/** Width of the bar of a stage in percent: full when done, the share of its units while it
 * counts, empty otherwise (a stage without a total has no honest share). */
function barWidth(run: Run | undefined, stage: Stage, state: StageState): number {
  const report = run?.stages[stage];
  if (state === STAGE_STATE.done) {
    return PERCENT;
  }
  if (report === undefined || report.total === 0) {
    return 0;
  }
  return Math.floor((report.done / report.total) * PERCENT);
}

function Row(props: { run: Run | undefined; stage: Stage }) {
  const state = () => (props.run ? stageState(props.run, props.stage) : STAGE_STATE.waiting);
  const took = () => (props.run ? stageSeconds(props.run, props.stage, lastEventAt()) : null);
  return (
    <div class={styles.row} {...{ [STATE_ATTRIBUTE]: state() }}>
      <span class={styles.mark}>{STAGE_MARKS[state()]}</span>
      <div>
        <div class={styles.name}>{t(STAGE_TEXT[props.stage].name)}</div>
        <div class={styles.detail}>{detail(props.run, props.stage)}</div>
      </div>
      <div class={styles.track}>
        <div
          class={styles.fill}
          style={{ width: barWidth(props.run, props.stage, state()) + PERCENT_SIGN }}
        />
      </div>
      <span class={styles.time}>
        <Show when={took() !== null}>
          {t(I18N_KEYS.SCAN_STAGE_TIME, { seconds: seconds(took() ?? 0, currentLanguage()) })}
        </Show>
      </span>
    </div>
  );
}

/** The six stages of the pipeline with state, detail, bar and the time each took (mock 5). The
 * times are measured in the browser between the events of the stream. */
export function Pipeline(props: { run: Run | undefined }) {
  return (
    <div class={styles.card}>
      <div class={styles.title}>{t(I18N_KEYS.SCAN_PIPELINE)}</div>
      <For each={STAGES}>{(stage) => <Row run={props.run} stage={stage} />}</For>
    </div>
  );
}
