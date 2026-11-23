import { FULL_SCAN_LOC } from "./consts";

/** The time of a scan of `loc` lines as the time per 100k lines, which is what the budget says.
 * A project below 100k lines (or without a line count) keeps the measured time, because scaling
 * a small scan up would make its fixed costs look like a slow scan. */
export function perFullScanUnit(ms: number, loc: number): number {
  return loc > FULL_SCAN_LOC ? (ms * FULL_SCAN_LOC) / loc : ms;
}
