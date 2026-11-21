import { createMemo, onMount, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { measuredTimes } from "../../shared/api/timings";
import { activeProject } from "../../shared/project/active";
import { LEVEL2_STAGE, SCAN_MODE } from "../projects/consts";
import { languageName, seconds } from "../projects/format";
import { level2RunOf } from "../projects/level2run";
import { projectAdding, projects, refreshProjects, scanRequestedAt } from "../projects/projects";
import type { ProjectRow } from "../projects/types";
import { refreshStatus, statusInfo } from "../status/status";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { Budgets, type Measured } from "./Budgets";
import { cancelScan } from "./cancel";
import {
  BRANCH_COMMIT_JOIN,
  CANCEL_KEY,
  FULL_SCAN_BUDGET,
  FULL_SCAN_LOC,
  IDLE,
  MS_PER_SECOND,
  SHORT_COMMIT_LENGTH,
  STATE,
} from "./consts";
import { scanRuns } from "./events";
import { Languages } from "./Languages";
import { Pipeline } from "./Pipeline";
import { Progress } from "./Progress";
import { Recent } from "./Recent";
import { latestRun } from "./run";
import type { BudgetTargets, Run } from "./types";
import styles from "./ScanScreen.module.css";

const TITLE: Record<Run["state"] | typeof IDLE, MessageKey> = {
  running: I18N_KEYS.SCAN_STATE_RUNNING,
  done: I18N_KEYS.SCAN_STATE_DONE,
  cancelled: I18N_KEYS.SCAN_STATE_CANCELLED,
  failed: I18N_KEYS.SCAN_STATE_FAILED,
  [IDLE]: I18N_KEYS.SCAN_STATE_IDLE,
};

/** The project the screen is about: the one the user opened. */
function subjectId(): number | null {
  return activeProject();
}

/** The mode a scan has: the first scan of a project is full, later ones are incremental. */
function modeText(row: ProjectRow | undefined, run: Run | undefined): string {
  const scanned = row?.scan != null;
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

/** What this session measured of the budgets. */
function measured(row: ProjectRow | undefined, run: Run | undefined): Measured {
  const result: Measured = { ...measuredTimes() };
  const start = statusInfo()?.startup_ms;
  if (start != null) {
    result.cold_start = start;
  }
  const took = duration(run);
  if (run?.state === STATE.done && row?.scan?.mode === SCAN_MODE.incremental && took !== null) {
    result.incremental_rescan = took * MS_PER_SECOND;
  }
  if (run?.state === STATE.done && row?.scan?.mode === SCAN_MODE.full && took !== null) {
    result.full_scan_100k_loc = took * MS_PER_SECOND;
  }
  return result;
}

/** The budgets whose measured value says less than the target: the full scan budget is for
 * 100k lines, so the scan of a smaller project is shown but not rated. */
function unrated(row: ProjectRow | undefined): Array<keyof BudgetTargets> {
  return row !== undefined && row.loc < FULL_SCAN_LOC ? [FULL_SCAN_BUDGET] : [];
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

/** Whether the scan runs, and the way to cancel it (button and key). */
function useCancel(run: () => Run | undefined) {
  const running = () => run()?.state === STATE.running;
  const cancel = () => {
    const id = subjectId();
    if (running() && id !== null) {
      void cancelScan(id);
    }
  };
  useKeydown((event) => {
    if (event.key === CANCEL_KEY && !event.defaultPrevented && !isTyping(event.target)) {
      cancel();
    }
  });
  return { running, cancel };
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

/** The head of the screen: title, meta line and the cancel button while a scan runs. */
function Head(props: {
  row: ProjectRow | undefined;
  run: Run | undefined;
  running: boolean;
  starting: boolean;
  onCancel: () => void;
}) {
  const title = () =>
    props.run === undefined && props.starting
      ? I18N_KEYS.SCAN_STATE_STARTING
      : TITLE[props.run?.state ?? IDLE];
  return (
    <div class={styles.head}>
      <h1>{t(title())}</h1>
      <Show when={props.row}>{(project) => <Meta project={project()} run={props.run} />}</Show>
      <Show when={props.running}>
        <button type="button" class={styles.cancel} onClick={props.onCancel}>
          {t(I18N_KEYS.SCAN_ACTION_CANCEL)}{" "}
          <kbd class={styles.kbd}>{t(I18N_KEYS.SCAN_CANCEL_KEY)}</kbd>
        </button>
      </Show>
    </div>
  );
}

/** The project of the screen, its newest run, and what of it the screen shows: while a new scan
 * has told nothing, the run of the scan before it is not shown as if it were this one. */
function useScan() {
  const row = createMemo(() => projects().find((item) => item.id === subjectId()));
  const run = createMemo(() => latestRun(scanRuns(), subjectId()));
  const starting = () => isStarting(row(), run()) || (run() === undefined && projectAdding());
  const shown = () => (starting() ? undefined : run());
  return { row, run, starting, shown };
}

/** The budgets and the recent scans, beside the progress. */
function Aside(props: { row: ProjectRow | undefined; run: Run | undefined }) {
  return (
    <aside class={styles.aside}>
      <Budgets
        targets={statusInfo()?.budgets ?? null}
        measured={measured(props.row, props.run)}
        unrated={unrated(props.row)}
      />
      <Recent rows={projects()} />
    </aside>
  );
}

/** Screen 5, scan and performance (blueprint B.5, mock `Scan.dc.html`): the progress of a scan as
 * the event stream reports it, the pipeline stages, the languages of the project, the time
 * budgets and the recent scans. */
export function ScanScreen() {
  onMount(() => {
    void refreshProjects();
    void refreshStatus();
  });
  const { row, run, starting, shown } = useScan();
  const { running, cancel } = useCancel(run);
  return (
    <section class={styles.screen}>
      <div class={styles.main}>
        <Head
          row={row()}
          run={shown()}
          running={running()}
          starting={starting()}
          onCancel={cancel}
        />
        <Show
          when={shown()}
          fallback={<Idle hasProjects={projects().length > 0} starting={starting()} />}
        >
          {(current) => (
            <Progress run={current()} linesPerSecond={linesPerSecond(row(), current())} />
          )}
        </Show>
        <Show when={shown()}>{(current) => <Pipeline run={current()} />}</Show>
        <Show when={subjectId()}>{(id) => <Indexer project={id()} />}</Show>
        <Show when={(row()?.languages.length ?? 0) > 0}>
          <Languages languages={row()?.languages ?? []} />
        </Show>
      </div>
      <Aside row={row()} run={run()} />
    </section>
  );
}
