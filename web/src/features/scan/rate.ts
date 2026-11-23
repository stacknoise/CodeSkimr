import { FULL_SCAN_LOC } from "./consts";

/** The time of a scan of `loc` lines as the time per 100k lines, which is what the budget says;
 * a project without a line count keeps the plain time. */
export function perFullScanUnit(ms: number, loc: number): number {
  return loc > 0 ? (ms * FULL_SCAN_LOC) / loc : ms;
}
