import type { BudgetTargets } from "../scan/types";

/** The output of `status.info`. */
export interface StatusInfo {
  /** Milliseconds from the start of the program until the server answered. */
  startup_ms: number | null;
  /** Memory of the program in megabytes. */
  memory_mb: number | null;
  /** The time budgets of blueprint 16.1, in milliseconds. */
  budgets: BudgetTargets;
  /** Threads the analyze stage of a scan uses: one per core the system offers. */
  workers: number;
}
