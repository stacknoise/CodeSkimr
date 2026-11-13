/** The output of `status.info`. */
export interface StatusInfo {
  /** Milliseconds from the start of the program until the server answered. */
  startup_ms: number | null;
  /** Memory of the program in megabytes. */
  memory_mb: number | null;
}
