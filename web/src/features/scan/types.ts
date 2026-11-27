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
      /** Label of the step of writing the snapshot, while the stage `persist` is in one. */
      step?: string;
      /** Place of that step in the write, counted from one. */
      step_number?: number;
      /** How many steps a write has at most. */
      steps?: number;
    }
  | {
      type: typeof EVENT.created;
      project: number;
      snapshot: number;
      /** Milliseconds that loading language packs took; the scan time leaves them out. */
      pack_load_ms?: number;
      /** Whether the scan found no file changed and only confirmed the snapshot (ADR 0093). */
      unchanged?: boolean;
    }
  | { type: typeof EVENT.changed; project: number; files: number; rescan: boolean }
  | {
      type: typeof EVENT.stopped;
      project: number;
      reason: (typeof STOP)[keyof typeof STOP];
      code: string | null;
    };

/** The step of writing the snapshot that the stage `persist` reported last. */
export interface StepRun {
  name: string;
  number: number;
  of: number;
  /** When the step began, in milliseconds of the clock of the browser. */
  startedAt: number;
}

/** What one stage of a run has reported. Times are in milliseconds of the clock of the browser. */
export interface StageRun {
  done: number;
  total: number;
  startedAt: number;
  endedAt: number | null;
  /** What the server measured for the stage; `null` while it has told nothing. */
  tookMs: number | null;
  /** The step of the write the stage is in; `null` outside a step. */
  step: StepRun | null;
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
  /** Milliseconds the server spent loading language packs during the run (ADR 0059). */
  packLoadMs: number;
  /** Whether the run ended without a change: the server confirmed the snapshot it had. */
  unchanged: boolean;
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

/** One scan of the list of the last scans (`scans.list`, ADR 0097). */
export interface ScanEntry {
  /** The snapshot the scan wrote, or confirmed if it found nothing changed; scans without a change
   * share the snapshot of the scan before them. */
  snapshot: number;
  /** Identifies the scan itself; unlike `snapshot` it is unique in the list. */
  scan: number;
  /** Seconds since the Unix epoch. */
  created: number;
  commit: string | null;
  mode: "full" | "incremental";
  files: number;
  symbols: number;
  /** Milliseconds; `null` for a scan stored before the duration was kept. */
  duration_ms: number | null;
  /** Files the scan changed against the one before; `null` for the first scan and for old ones. */
  files_changed: number | null;
}

/** The output of `scans.list`. */
export interface ScanHistoryAnswer {
  scans: ScanEntry[];
  /** How many scans the project keeps. */
  keep: number;
  /** Average duration in milliseconds of the scans that have one. */
  average_ms: number | null;
}
