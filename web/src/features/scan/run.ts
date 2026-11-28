import {
  EVENT,
  CHECK_STAGE,
  FILE_STAGE,
  MAX_RUNS,
  MIN_FILES_FOR_ESTIMATE,
  NOT_FOUND,
  MS_PER_SECOND,
  LAST_RUNNING_PERCENT,
  PERCENT,
  STAGE_STATE,
  STAGES,
  STEP_STAGE,
  SECONDS_PER_MINUTE,
  CLOCK_SECONDS_WIDTH,
  STATE,
  STOP,
} from "./consts";
import type { Run, Stage, StageRun, StepRun, StepTime, WireEvent } from "./types";

type ProgressEvent = Extract<WireEvent, { type: typeof EVENT.progressed }>;

/** Closes every stage that is open: it ended at `now`. */
function closeStages(stages: Run["stages"], now: number): Run["stages"] {
  const closed: Run["stages"] = {};
  for (const stage of STAGES) {
    const run = stages[stage];
    if (run) {
      closed[stage] = run.endedAt === null ? { ...run, endedAt: now } : run;
    }
  }
  return closed;
}

/** The step that a report names, with the time it began: the time of the report when it is a new
 * step, the earlier time while the same step goes on. `null` for a report without a step. */
function stepOf(event: ProgressEvent, known: StageRun | undefined, now: number): StepRun | null {
  if (event.step === undefined || event.step_number === undefined || event.steps === undefined) {
    return null;
  }
  const before = known?.step;
  return {
    name: event.step,
    number: event.step_number,
    of: event.steps,
    startedAt: before?.name === event.step ? before.startedAt : now,
  };
}

/** The stage of `stage` after a report: earlier stages end when a later one starts. */
function advance(run: Run, event: ProgressEvent, now: number): Run {
  const { stage, done, total } = event;
  const index = STAGES.indexOf(stage);
  const stages: Run["stages"] = {};
  for (const name of STAGES) {
    const known = run.stages[name];
    if (STAGES.indexOf(name) < index && known) {
      stages[name] = known.endedAt === null ? { ...known, endedAt: now } : known;
    } else if (name === stage) {
      stages[name] = {
        // A report that was queued before the stream replayed a later one must not move the bar
        // back: within one stage and one total the count only grows.
        done: known && known.total === total ? Math.max(known.done, done) : done,
        total,
        startedAt: known?.startedAt ?? now - (event.elapsed_ms ?? 0),
        endedAt: null,
        tookMs: Math.max(known?.tookMs ?? 0, event.elapsed_ms ?? 0) || null,
        step: stepOf(event, known, now),
      };
    } else if (known) {
      stages[name] = known;
    }
  }
  return { ...run, stages };
}

/** A run that begins with the report `first`. A report that comes from a stage that began
 * earlier (a page that was reloaded in the middle of a scan) puts the start back by the time the
 * stage has run, so that the times and rates are those of the scan and not of the page. */
function fresh(project: number, now: number, first: ProgressEvent): Run {
  return {
    project,
    startedAt: now - (first.elapsed_ms ?? 0),
    endedAt: null,
    state: STATE.running,
    code: null,
    stages: {},
    packLoadMs: 0,
    unchanged: false,
  };
}

function isOpen(run: Run | undefined): run is Run {
  return run?.state === STATE.running;
}

function lastOf(runs: readonly Run[], project: number): number {
  for (let at = runs.length - 1; at >= 0; at -= 1) {
    if (runs[at]?.project === project) {
      return at;
    }
  }
  return NOT_FOUND;
}

function replace(runs: readonly Run[], index: number, run: Run): Run[] {
  return runs.map((old, at) => (at === index ? run : old));
}

/** Folds one event of the stream into the list of runs. Pure: `now` is the clock of the browser.
 * An event of a stage starts a run if the project has none that is running. */
export function applyEvent(runs: readonly Run[], event: WireEvent, now: number): Run[] {
  if (event.type === EVENT.changed) {
    return [...runs];
  }
  const index = lastOf(runs, event.project);
  const current = isOpen(runs[index]) ? runs[index] : undefined;
  if (event.type === EVENT.progressed) {
    const base = current ?? fresh(event.project, now, event);
    const next = advance(base, event, now);
    const list = current ? replace(runs, index, next) : [...runs, next];
    return list.slice(-MAX_RUNS);
  }
  if (!current) {
    return [...runs];
  }
  if (event.type === EVENT.created) {
    const stages = closeStages(current.stages, now);
    const packLoadMs = event.pack_load_ms ?? 0;
    return replace(runs, index, {
      ...current,
      state: STATE.done,
      endedAt: now,
      stages,
      packLoadMs,
      unchanged: event.unchanged === true,
    });
  }
  const state = event.reason === STOP.cancelled ? STATE.cancelled : STATE.failed;
  return replace(runs, index, { ...current, state, code: event.code, endedAt: now });
}

/** The newest run of a project, if the stream told of one. */
export function latestRun(runs: readonly Run[], project: number | null): Run | undefined {
  return project === null ? undefined : runs[lastOf(runs, project)];
}

/** The report of the stage that counts files. */
function fileStage(run: Run): StageRun | undefined {
  return run.stages[FILE_STAGE];
}

/** Files done and files in total; `null` before the file stage reported. */
export function files(run: Run): { done: number; total: number } | null {
  const stage = fileStage(run);
  return stage ? { done: stage.done, total: stage.total } : null;
}

/** The number of files a scan compared with the snapshot when the server said that none changed;
 * `null` for any other run. */
export function checkedOnly(run: Run): number | null {
  const found = run.stages[CHECK_STAGE];
  return run.unchanged && found !== undefined ? found.total : null;
}

/** Percent of the files that are done, at most 99 while the scan runs. 100 once the scan is done; 0
 * before the file stage. */
export function percent(run: Run): number {
  if (run.state === STATE.done) {
    return PERCENT;
  }
  const stage = fileStage(run);
  if (!stage) {
    return 0;
  }
  const share = stage.total === 0 ? 0 : Math.floor((stage.done / stage.total) * PERCENT);
  // The later stages still have to run: a scan says 100 only when it is done.
  return Math.min(stage.endedAt !== null ? PERCENT : share, LAST_RUNNING_PERCENT);
}

/** Files per second since the file stage began (until it ended), or `null` if not measurable. */
export function filesPerSecond(run: Run, now: number): number | null {
  const stage = fileStage(run);
  if (!stage) {
    return null;
  }
  const seconds = ((stage.endedAt ?? run.endedAt ?? now) - stage.startedAt) / MS_PER_SECOND;
  return seconds > 0 && stage.done > 0 ? stage.done / seconds : null;
}

/** Seconds left in the file stage at the current rate; `null` when the rate is not yet reliable
 * or the stage is over. */
export function secondsLeft(run: Run, now: number): number | null {
  const stage = fileStage(run);
  const rate = filesPerSecond(run, now);
  if (
    !stage ||
    stage.endedAt !== null ||
    run.endedAt !== null ||
    rate === null ||
    stage.done < MIN_FILES_FOR_ESTIMATE
  ) {
    return null;
  }
  return Math.max(0, (stage.total - stage.done) / rate);
}

/** Seconds a stage took, or `null` if it has not started. A stage that is over has the time the
 * server measured; one that still runs has the time since its first report, and a server that
 * sends no time leaves the time between the events as the browser saw them. */
export function stageSeconds(run: Run, stage: Stage, now: number): number | null {
  const report = run.stages[stage];
  if (report === undefined) {
    return null;
  }
  const over = report.endedAt !== null || run.endedAt !== null;
  if (over && report.tookMs !== null) {
    return report.tookMs / MS_PER_SECOND;
  }
  return ((report.endedAt ?? run.endedAt ?? now) - report.startedAt) / MS_PER_SECOND;
}

/** How a stage stands in a run. */
export type StageState = (typeof STAGE_STATE)[keyof typeof STAGE_STATE];

/** A stage is done once it ended or the whole scan is done; it runs while it reports; the stage
 * that was open when the scan was cancelled or failed is stopped. */
export function stageState(run: Run, stage: Stage): StageState {
  const report = run.stages[stage];
  if (run.state === STATE.done || (report !== undefined && report.endedAt !== null)) {
    return STAGE_STATE.done;
  }
  if (report === undefined) {
    return STAGE_STATE.waiting;
  }
  return run.state === STATE.running ? STAGE_STATE.running : STAGE_STATE.stopped;
}

/** Percent of one stage that is done: 100 once it is over, the share of its units while they
 * count, `null` while the stage waits or runs without a count that says anything (it has told
 * no total, or has not finished a unit yet). */
export function stagePercent(run: Run, stage: Stage): number | null {
  const state = stageState(run, stage);
  if (state === STAGE_STATE.done) {
    return PERCENT;
  }
  const report = run.stages[stage];
  if (report === undefined || report.total === 0 || report.done === 0) {
    return null;
  }
  return Math.floor((report.done / report.total) * PERCENT);
}

/** Whether a stage works but cannot say how far it has come: the bar then moves without a length
 * instead of claiming a share. */
export function isUnmeasured(run: Run, stage: Stage): boolean {
  return stageState(run, stage) === STAGE_STATE.running && stagePercent(run, stage) === null;
}

/** The step of the write that the stage `persist` is in right now, with the seconds it has taken
 * so far; `null` while the scan is not in a step. */
export function activeStep(run: Run, now: number): { step: StepRun; seconds: number } | null {
  const report = run.stages[STEP_STAGE];
  if (run.state !== STATE.running || report === undefined || report.endedAt !== null) {
    return null;
  }
  const step = report.step;
  return step === null
    ? null
    : { step, seconds: Math.max(0, now - step.startedAt) / MS_PER_SECOND };
}

/** A time as "4:12": minutes and seconds, the seconds with two digits. */
export function minutesSeconds(value: number): string {
  const whole = Math.floor(value);
  const rest = String(whole % SECONDS_PER_MINUTE).padStart(CLOCK_SECONDS_WIDTH, "0");
  return `${Math.floor(whole / SECONDS_PER_MINUTE)}:${rest}`;
}

/** The milliseconds of all steps together. */
export function totalMs(steps: readonly StepTime[]): number {
  return steps.reduce((sum, time) => sum + time.duration_ms, 0);
}

/** The step that took longest, `undefined` without steps. */
export function slowestStep(steps: readonly StepTime[]): StepTime | undefined {
  return steps.reduce<StepTime | undefined>(
    (slowest, time) =>
      slowest === undefined || time.duration_ms > slowest.duration_ms ? time : slowest,
    undefined,
  );
}
