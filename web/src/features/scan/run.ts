import {
  EVENT,
  FILE_STAGE,
  MAX_RUNS,
  MIN_FILES_FOR_ESTIMATE,
  NOT_FOUND,
  MS_PER_SECOND,
  PERCENT,
  STAGE_STATE,
  STAGES,
  STATE,
  STOP,
} from "./consts";
import type { Run, Stage, StageRun, WireEvent } from "./types";

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

/** The stage of `stage` after a report: earlier stages end when a later one starts. */
function advance(run: Run, stage: Stage, done: number, total: number, now: number): Run {
  const index = STAGES.indexOf(stage);
  const stages: Run["stages"] = {};
  for (const name of STAGES) {
    const known = run.stages[name];
    if (STAGES.indexOf(name) < index && known) {
      stages[name] = known.endedAt === null ? { ...known, endedAt: now } : known;
    } else if (name === stage) {
      stages[name] = { done, total, startedAt: known?.startedAt ?? now, endedAt: null };
    } else if (known) {
      stages[name] = known;
    }
  }
  return { ...run, stages };
}

function fresh(project: number, now: number): Run {
  return { project, startedAt: now, endedAt: null, state: STATE.running, code: null, stages: {} };
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
    const base = current ?? fresh(event.project, now);
    const next = advance(base, event.stage, event.done, event.total, now);
    const list = current ? replace(runs, index, next) : [...runs, next];
    return list.slice(-MAX_RUNS);
  }
  if (!current) {
    return [...runs];
  }
  if (event.type === EVENT.created) {
    const stages = closeStages(current.stages, now);
    return replace(runs, index, { ...current, state: STATE.done, endedAt: now, stages });
  }
  const state = event.reason === STOP.cancelled ? STATE.cancelled : STATE.failed;
  return replace(runs, index, { ...current, state, code: event.code, endedAt: now });
}

/** The newest run of a project, if the stream told of one. */
export function latestRun(runs: readonly Run[], project: number | null): Run | undefined {
  return project === null ? undefined : runs[lastOf(runs, project)];
}

/** The project of the run that is running now, else of the newest run. */
export function focusProject(runs: readonly Run[]): number | null {
  const running = runs.filter((run) => run.state === STATE.running);
  return (running[running.length - 1] ?? runs[runs.length - 1])?.project ?? null;
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

/** Percent of the files that are done. 100 once the scan is done; 0 before the file stage. */
export function percent(run: Run): number {
  if (run.state === STATE.done) {
    return PERCENT;
  }
  const stage = fileStage(run);
  if (!stage) {
    return 0;
  }
  if (stage.endedAt !== null) {
    return PERCENT;
  }
  return stage.total === 0 ? 0 : Math.floor((stage.done / stage.total) * PERCENT);
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

/** Seconds a stage took so far, or `null` if it has not started. */
export function stageSeconds(run: Run, stage: Stage, now: number): number | null {
  const report = run.stages[stage];
  return report
    ? ((report.endedAt ?? run.endedAt ?? now) - report.startedAt) / MS_PER_SECOND
    : null;
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
