import { createMemo, onMount, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { LEVEL2_STAGE, SCAN_MODE } from "../projects/consts";
import { languageName, seconds } from "../projects/format";
import { ScanButtons, ScanMessages } from "../projects/ScanPanel";
import { level2RunOf } from "../projects/level2run";
import { projectAdding, scanRequestedAt } from "../projects/projects";
import type { ProjectRow } from "../projects/types";
import { refreshStatus } from "../status/status";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { cancelScan } from "./cancel";
import {
  BRANCH_COMMIT_JOIN,
  CANCEL_KEY,
  IDLE,
  MS_PER_SECOND,
  SHORT_COMMIT_LENGTH,
  STATE,
} from "./consts";
import { scanRuns } from "./events";
import { Languages } from "./Languages";
import { LeftOut } from "./LeftOut";
import { Pipeline } from "./Pipeline";
import { Progress } from "./Progress";
import { latestRun } from "./run";
import type { Run } from "./types";
import styles from "./ScanTab.module.css";

const TITLE: Record<Run["state"] | typeof IDLE, MessageKey> = {
  running: I18N_KEYS.SCAN_STATE_RUNNING,
  done: I18N_KEYS.SCAN_STATE_DONE,
  cancelled: I18N_KEYS.SCAN_STATE_CANCELLED,
  failed: I18N_KEYS.SCAN_STATE_FAILED,
  [IDLE]: I18N_KEYS.SCAN_STATE_IDLE,
};

/** The mode a scan has: the first scan of a project is full, later ones are incremental. */
function modeText(row: ProjectRow | undefined, run: Run | undefined): string {
  const scanned = row?.scan != null;
  if (run?.state === STATE.running && !scanned) {
    return t(I18N_KEYS.SCAN_MODE_FIRST);
  }
  const incremental =
    run?.state === STATE.running ? scanned : row?.scan?.mode === SCAN_MODE.incremental;
  return t(incremental ? I18N_KEYS.SCAN_MODE_INCREMENTAL : I18N_KEYS.SCAN_MODE_FULL);
}

/** The place of the scan in the history of the source: `main @ e07c4a1`, or what is known of it. */
export function positionText(row: ProjectRow): string | null {
  const commit = row.commit?.slice(0, SHORT_COMMIT_LENGTH);
  const parts = [row.branch, commit].filter((part): part is string => !!part);
  return parts.length === 0 ? null : parts.join(BRANCH_COMMIT_JOIN);
}

function duration(run: Run | undefined): number | null {
  return run?.endedAt == null
    ? null
    : Math.max(run.endedAt - run.startedAt - run.packLoadMs, 0) / MS_PER_SECOND;
}

/** Lines per second of a finished full scan, else `null`. */
function linesPerSecond(row: ProjectRow | undefined, run: Run | undefined): number | null {
  const took = duration(run);
  const full = row?.scan?.mode === SCAN_MODE.full;
  return run?.state === STATE.done && full && took !== null && took > 0 ? row.loc / took : null;
}

/** What the screen says while it has no report of a scan: that a scan was started and has told
 * nothing yet, or that none runs. */
function Idle(props: { hasProjects: boolean; starting: boolean }) {
  const hint = () => {
    if (props.starting) {
      return I18N_KEYS.SCAN_STARTING_HINT;
    }
    return props.hasProjects ? I18N_KEYS.SCAN_IDLE_HINT : I18N_KEYS.SCAN_NO_PROJECT;
  };
  return <div class={styles.idle}>{t(hint())}</div>;
}

/** Whether a scan of the project was started and has told nothing yet: the list says it runs
 * or this tab asked for it, and the newest run the stream knows is older than that. */
function isStarting(row: ProjectRow | undefined, run: Run | undefined): boolean {
  if (row === undefined) {
    return false;
  }
  if (run === undefined) {
    return row.running || scanRequestedAt(row.id) !== null;
  }
  const asked = scanRequestedAt(row.id);
  return run.state !== STATE.running && asked !== null && run.startedAt < asked;
}

/** The pass of stage 2 that works for the project, as far as the stream told; it has no count. */
function Indexer(props: { project: number }) {
  return (
    <Show when={level2RunOf(props.project)}>
      {(run) => (
        <div class={styles.indexer}>
          {t(
            run().stage === LEVEL2_STAGE.importing
              ? I18N_KEYS.PROJECTS_LEVEL2_RUNNING_IMPORTING
              : I18N_KEYS.PROJECTS_LEVEL2_RUNNING,
            { language: languageName(run().language) },
          )}
        </div>
      )}
    </Show>
  );
}

/** What the scan is about: project, position, mode and how long it took. */
function Meta(props: { project: ProjectRow; run: Run | undefined }) {
  return (
    <span class={styles.meta}>
      {props.project.name} ·{" "}
      <Show when={positionText(props.project)}>{(place) => <>{place()} · </>}</Show>
      {modeText(props.project, props.run)}
      <Show when={duration(props.run)}>
        {(took) => (
          <>
            {" "}
            ·{" "}
            {t(I18N_KEYS.SCAN_STAGE_TIME, {
              seconds: seconds(took(), currentLanguage()),
            })}
          </>
        )}
      </Show>
    </span>
  );
}

/** The title and the meta line of the tab: how the scan stands and what it is about. */
function Head(props: { row: ProjectRow; run: Run | undefined; starting: boolean }) {
  const title = () =>
    props.run === undefined && props.starting
      ? I18N_KEYS.SCAN_STATE_STARTING
      : TITLE[props.run?.state ?? IDLE];
  return (
    <div class={styles.head}>
      <h2>{t(title())}</h2>
      <Meta project={props.row} run={props.run} />
    </div>
  );
}

/** The newest run of a project, and what of it is shown: while a new scan has told nothing, the
 * run of the scan before it is not shown as if it were this one. */
export function useScanOf(project: () => ProjectRow | undefined) {
  const run = createMemo(() => latestRun(scanRuns(), project()?.id ?? null));
  const starting = () => isStarting(project(), run()) || (run() === undefined && projectAdding());
  const shown = () => (starting() ? undefined : run());
  const running = () => shown()?.state === STATE.running;
  return { run, starting, shown, running };
}

/** Cancels the scan of a project on the key Esc, when no input or question is open (they take
 * the key first). */
export function useCancelKey(
  project: () => number | null,
  running: () => boolean,
  free: () => boolean,
) {
  useKeydown((event) => {
    const id = project();
    const plain = event.key === CANCEL_KEY && !event.defaultPrevented && !isTyping(event.target);
    if (plain && running() && free() && id !== null) {
      void cancelScan(id);
    }
  });
}

/** The tab "Scan" of the project page (blueprint B.5, mock 23): the progress of a scan as the event
 * stream reports it, the steps, the languages of the project and the files left out. */
export function ScanTab(props: { row: ProjectRow }) {
  onMount(() => void refreshStatus());
  const { starting, shown } = useScanOf(() => props.row);
  return (
    <div class={styles.main}>
      <Head row={props.row} run={shown()} starting={starting()} />
      <ScanButtons row={props.row} />
      <ScanMessages row={props.row} />
      <Show when={shown()} fallback={<Idle hasProjects={true} starting={starting()} />}>
        {(current) => (
          <Progress run={current()} linesPerSecond={linesPerSecond(props.row, current())} />
        )}
      </Show>
      <Show when={shown()}>{(current) => <Pipeline run={current()} />}</Show>
      <Indexer project={props.row.id} />
      <Show when={props.row.languages.length > 0}>
        <Languages languages={props.row.languages} />
      </Show>
      <LeftOut row={props.row} />
    </div>
  );
}
