import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { SCAN_MODE } from "../projects/consts";
import { moment } from "../scan/moment";
import type { ScanEntry } from "../scan/types";
import { COMMIT_JOIN, PART_SEPARATOR, SHORT_COMMIT } from "./consts";

/** The first characters of the commit of a scan, `null` for a scan of a folder. */
function shortCommit(scan: ScanEntry): string | null {
  return scan.commit === null ? null : scan.commit.slice(0, SHORT_COMMIT);
}

/** The scan as a small chip names it: when it ran and, for a Git project, its commit. */
export function chipText(scan: ScanEntry): string {
  const commit = shortCommit(scan);
  return commit === null ? moment(scan.created) : `${moment(scan.created)}${COMMIT_JOIN}${commit}`;
}

/** The scan as the choice of a selector names it: "Scan of <time> · full · commit"; the newest
 * scan is "Current scan" with its time. */
export function optionText(scan: ScanEntry, newest: boolean): string {
  const mode = t(
    scan.mode === SCAN_MODE.incremental
      ? I18N_KEYS.SCAN_MODE_INCREMENTAL
      : I18N_KEYS.SCAN_MODE_FULL,
  );
  const commit = shortCommit(scan);
  const first = newest
    ? t(I18N_KEYS.CHANGE_VIEW_SCAN_CURRENT)
    : t(I18N_KEYS.CHANGE_VIEW_SCAN_OPTION, { time: moment(scan.created) });
  return [first, mode, ...(commit === null ? [] : [commit])].join(PART_SEPARATOR);
}
