import { createResource, createSignal, For, onCleanup, Show } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, seconds } from "../projects/format";
import {
  COMMAND_SCANS_LIST,
  FILE_STAGE,
  PERCENT,
  NOTE_ID_PREFIX,
  PERCENT_SIGN,
  PROGRESS_MAX,
  PROGRESS_MIN,
  STAGE_MARKS,
  STAGE_STATE,
  STAGE_TEXT,
  STAGES,
  STATE,
  STATE_ATTRIBUTE,
  STEP_STAGE,
  STEP_TEXT,
  STEP_TICK_MS,
} from "./consts";
import { lastEventAt } from "./events";
import {
  activeStep,
  isUnmeasured,
  minutesSeconds,
  stagePercent,
  stageSeconds,
  stageState,
  type StageState,
} from "./run";
import type { Run, ScanHistoryAnswer, Stage } from "./types";
import { StepTimes } from "./StepTimes";
import { cx } from "../../shared/ui/cx";
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
  if (state === STAGE_STATE.done) {
    return PERCENT;
  }
  return run === undefined ? 0 : (stagePercent(run, stage) ?? 0);
}

/** The id of the note that says why a stage has no share. */
function noteId(stage: Stage): string {
  return `${NOTE_ID_PREFIX}${stage}`;
}

/** The bar of a stage: its share, or a moving bar for a stage that cannot say how far it is. */
function Bar(props: { run: Run | undefined; stage: Stage; state: StageState }) {
  const share = () => (props.run ? stagePercent(props.run, props.stage) : null);
  const unmeasured = () => (props.run ? isUnmeasured(props.run, props.stage) : false);
  return (
    <div
      class={styles.track}
      role="progressbar"
      aria-label={t(STAGE_TEXT[props.stage].name)}
      aria-valuemin={PROGRESS_MIN}
      aria-valuemax={PROGRESS_MAX}
      aria-valuenow={share() ?? undefined}
      aria-describedby={unmeasured() ? noteId(props.stage) : undefined}
    >
      <div
        class={cx(styles.fill, unmeasured() && styles.unmeasured)}
        style={
          unmeasured()
            ? undefined
            : { width: barWidth(props.run, props.stage, props.state) + PERCENT_SIGN }
        }
      />
    </div>
  );
}

/** The step of the write the stage is in, with its place and the time it has taken so far. The
 * time runs on a timer of its own: a step that takes minutes sends nothing in between. */
function StepLine(props: { run: Run; fallback: string }) {
  const [now, setNow] = createSignal(Date.now());
  const timer = setInterval(() => setNow(Date.now()), STEP_TICK_MS);
  onCleanup(() => clearInterval(timer));
  const active = () => activeStep(props.run, now());
  const key = () => STEP_TEXT[active()?.step.name ?? ""];
  return (
    <Show when={active() !== null && key() !== undefined} fallback={props.fallback}>
      {t(I18N_KEYS.SCAN_STEP_LINE, {
        step: t(key() ?? I18N_KEYS.SCAN_STEP_HEADER),
        number: active()?.step.number ?? 0,
        of: active()?.step.of ?? 0,
        time: minutesSeconds(active()?.seconds ?? 0),
      })}
    </Show>
  );
}

function Row(props: { run: Run | undefined; stage: Stage }) {
  const state = () => (props.run ? stageState(props.run, props.stage) : STAGE_STATE.waiting);
  const share = () => (props.run ? stagePercent(props.run, props.stage) : null);
  const unmeasured = () => (props.run ? isUnmeasured(props.run, props.stage) : false);
  const took = () => (props.run ? stageSeconds(props.run, props.stage, lastEventAt()) : null);
  return (
    <div class={styles.row} {...{ [STATE_ATTRIBUTE]: state() }}>
      <span class={styles.mark}>{STAGE_MARKS[state()]}</span>
      <div>
        <div class={styles.name}>{t(STAGE_TEXT[props.stage].name)}</div>
        <div class={styles.detail}>
          <Show
            when={props.stage === STEP_STAGE && props.run}
            fallback={detail(props.run, props.stage)}
          >
            {(run) => <StepLine run={run()} fallback={detail(run(), props.stage)} />}
          </Show>
        </div>
        <Show when={unmeasured()}>
          <div class={styles.detail} id={noteId(props.stage)}>
            {t(I18N_KEYS.SCAN_STAGE_UNMEASURED)}
          </div>
        </Show>
      </div>
      <Bar run={props.run} stage={props.stage} state={state()} />
      <span class={styles.share}>
        <Show when={share() !== null}>
          {share()} {PERCENT_SIGN}
        </Show>
      </span>
      <span class={styles.time}>
        <Show when={took() !== null}>
          {t(I18N_KEYS.SCAN_STAGE_TIME, { seconds: seconds(took() ?? 0, currentLanguage()) })}
        </Show>
      </span>
    </div>
  );
}

/** The times of the steps of the write for a scan that is over and wrote a snapshot, read from the
 * list of the last scans once the scan has ended: the server keeps them with the scan. */
function SavedSteps(props: { run: Run }) {
  const source = () =>
    props.run.state === STATE.done && !props.run.unchanged
      ? { project: props.run.project, endedAt: props.run.endedAt }
      : undefined;
  const [answer] = createResource(source, (input) =>
    runCommand<ScanHistoryAnswer>(COMMAND_SCANS_LIST, { project: input.project }),
  );
  return <StepTimes steps={answer.latest?.scans[0]?.steps ?? []} />;
}

/** The six stages of the pipeline with state, detail, bar and the time each took (mock 5). The
 * times are measured in the browser between the events of the stream. */
export function Pipeline(props: { run: Run | undefined }) {
  return (
    <div class={styles.card}>
      <div class={styles.title}>{t(I18N_KEYS.SCAN_PIPELINE)}</div>
      <For each={STAGES}>{(stage) => <Row run={props.run} stage={stage} />}</For>
      <Show when={props.run}>{(run) => <SavedSteps run={run()} />}</Show>
    </div>
  );
}
