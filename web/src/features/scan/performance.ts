import { measuredTimes } from "../../shared/api/timings";
import { SCAN_MODE } from "../projects/consts";
import { selectedProject } from "../projects/page";
import { projects } from "../projects/projects";
import type { ProjectRow } from "../projects/types";
import { statusInfo } from "../status/status";
import type { Measured } from "./Budgets";
import { scanRuns } from "./events";
import { BUDGET_ROWS, FULL_SCAN_BUDGET, FULL_SCAN_LOC, MS_PER_SECOND, STATE } from "./consts";
import { perFullScanUnit } from "./rate";
import { latestRun } from "./run";
import type { BudgetTargets, Run } from "./types";

/** Seconds a finished run took, without the loading of the language packs; `null` while open. */
export function duration(run: Run | undefined): number | null {
  return run?.endedAt == null
    ? null
    : Math.max(run.endedAt - run.startedAt - run.packLoadMs, 0) / MS_PER_SECOND;
}

/** What this session measured of the budgets: the times of the commands, the start of the
 * program and, from the newest scan of the selected project, the scan budgets. */
export function measured(row: ProjectRow | undefined, run: Run | undefined): Measured {
  const result: Measured = { ...measuredTimes() };
  const start = statusInfo()?.startup_ms;
  if (start != null) {
    result.cold_start = start;
  }
  const took = duration(run);
  if (run?.state === STATE.done && took !== null && row?.scan?.mode === SCAN_MODE.incremental) {
    result.incremental_rescan = took * MS_PER_SECOND;
  }
  if (run?.state === STATE.done && took !== null && row?.scan?.mode === SCAN_MODE.full) {
    result.full_scan_100k_loc = perFullScanUnit(took * MS_PER_SECOND, row.loc);
  }
  return result;
}

/** The budgets that are shown but not rated: the full scan budget is for 100k lines, so the scan of
 * a smaller project says nothing about it. */
export function unrated(row: ProjectRow | undefined): Array<keyof BudgetTargets> {
  return row !== undefined && row.loc < FULL_SCAN_LOC ? [FULL_SCAN_BUDGET] : [];
}

/** The state of the performance targets of the whole app: what was measured, what is not rated and
 * how many targets were missed. */
export function performance() {
  const row = projects().find((item) => item.id === selectedProject());
  const run = latestRun(scanRuns(), row?.id ?? null);
  const values = measured(row, run);
  const skipped = unrated(row);
  const targets = statusInfo()?.budgets ?? null;
  const missed = BUDGET_ROWS.filter((budget) => {
    const value = values[budget.key];
    return (
      targets !== null &&
      value !== undefined &&
      !skipped.includes(budget.key) &&
      value > targets[budget.key]
    );
  }).length;
  return { targets, measured: values, unrated: skipped, missed };
}
