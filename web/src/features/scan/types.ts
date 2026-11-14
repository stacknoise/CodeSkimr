import type { EVENT, STAGES, STATE, STOP } from "./consts";

/** A stage name. */
export type Stage = (typeof STAGES)[number];

/** One event of the stream. */
export type WireEvent =
  | {
      type: typeof EVENT.progressed;
      project: number;
      stage: Stage;
      done: number;
      total: number;
      /** Milliseconds since the stage started, as the server measured them. */
      elapsed_ms?: number;
    }
  | { type: typeof EVENT.created; project: number; snapshot: number }
  | { type: typeof EVENT.changed; project: number; files: number; rescan: boolean }
  | {
      type: typeof EVENT.stopped;
      project: number;
      reason: (typeof STOP)[keyof typeof STOP];
      code: string | null;
    };

/** What one stage of a run has reported. Times are in milliseconds of the clock of the browser. */
export interface StageRun {
  done: number;
  total: number;
  startedAt: number;
  endedAt: number | null;
  /** What the server measured for the stage; `null` while it has told nothing. */
  tookMs: number | null;
}

/** How a run stands. */
export type RunState = (typeof STATE)[keyof typeof STATE];

/** One scan of one project, as far as the stream told. */
export interface Run {
  project: number;
  startedAt: number;
  endedAt: number | null;
  state: RunState;
  /** The registered code of a failure. */
  code: string | null;
  stages: Partial<Record<Stage, StageRun>>;
}

/** The measurements of the budgets that `status.info` returns. */
export type BudgetTargets = Record<
  | "cold_start"
  | "search_keystroke"
  | "explorer_switch"
  | "usages_query"
  | "graph_layout"
  | "full_scan_100k_loc"
  | "incremental_rescan",
  number
>;
