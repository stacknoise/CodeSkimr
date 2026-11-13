import { createMemo, onMount, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { SCAN_MODE } from "../projects/consts";
import { seconds } from "../projects/format";
import { projects, refreshProjects } from "../projects/projects";
import type { ProjectRow } from "../projects/types";
import { refreshStatus, statusInfo } from "../status/status";
import { Budgets, type Measured } from "./Budgets";
import { STATE, MS_PER_SECOND } from "./consts";
import { scanRuns } from "./events";
import { Languages } from "./Languages";
import { Pipeline } from "./Pipeline";
import { Progress } from "./Progress";
import { Recent } from "./Recent";
import { focusProject, latestRun } from "./run";
import type { Run } from "./types";
import styles from "./ScanScreen.module.css";

const TITLE: Record<Run["state"] | "idle", MessageKey> = {
  running: I18N_KEYS.SCAN_STATE_RUNNING,
  done: I18N_KEYS.SCAN_STATE_DONE,
  cancelled: I18N_KEYS.SCAN_STATE_CANCELLED,
  failed: I18N_KEYS.SCAN_STATE_FAILED,
  idle: I18N_KEYS.SCAN_STATE_IDLE,
};

/** The project the screen is about: the one that is scanning, else the one the user opened, else
 * the first. */
function subjectId(): number | null {
  return focusProject(scanRuns()) ?? activeProject() ?? projects()[0]?.id ?? null;
}

/** The mode a scan has: the first scan of a project is full, later ones are incremental. */
function modeText(row: ProjectRow | undefined, run: Run | undefined): string {
  const scanned = row?.scan != null;
  const incremental =
    run?.state === STATE.running ? scanned : row?.scan?.mode === SCAN_MODE.incremental;
  return t(incremental ? I18N_KEYS.SCAN_MODE_INCREMENTAL : I18N_KEYS.SCAN_MODE_FULL);
}

function duration(run: Run | undefined): number | null {
  return run?.endedAt == null ? null : (run.endedAt - run.startedAt) / MS_PER_SECOND;
}

/** Lines per second of a finished full scan, else `null`. */
function linesPerSecond(row: ProjectRow | undefined, run: Run | undefined): number | null {
  const took = duration(run);
  const full = row?.scan?.mode === SCAN_MODE.full;
  return run?.state === STATE.done && full && took !== null && took > 0 ? row.loc / took : null;
}

/** What this session measured of the budgets. */
function measured(row: ProjectRow | undefined, run: Run | undefined): Measured {
  const result: Measured = {};
  const start = statusInfo()?.startup_ms;
  if (start != null) {
    result.cold_start = start;
  }
  const took = duration(run);
  if (run?.state === STATE.done && row?.scan?.mode === SCAN_MODE.incremental && took !== null) {
    result.incremental_rescan = took * MS_PER_SECOND;
  }
  return result;
}

function Idle(props: { hasProjects: boolean }) {
  return (
    <div class={styles.idle}>
      {t(props.hasProjects ? I18N_KEYS.SCAN_IDLE_HINT : I18N_KEYS.SCAN_NO_PROJECT)}
    </div>
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
  const row = createMemo(() => projects().find((item) => item.id === subjectId()));
  const run = createMemo(() => latestRun(scanRuns(), subjectId()));
  return (
    <section class={styles.screen}>
      <div class={styles.main}>
        <div class={styles.head}>
          <h1>{t(TITLE[run()?.state ?? "idle"])}</h1>
          <Show when={row()}>
            {(project) => (
              <span class={styles.meta}>
                {project().name} · {modeText(project(), run())}
                <Show when={duration(run())}>
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
            )}
          </Show>
        </div>
        <Show when={run()} fallback={<Idle hasProjects={projects().length > 0} />}>
          {(current) => (
            <Progress run={current()} linesPerSecond={linesPerSecond(row(), current())} />
          )}
        </Show>
        <Show when={run()}>{(current) => <Pipeline run={current()} />}</Show>
        <Show when={(row()?.languages.length ?? 0) > 0}>
          <Languages languages={row()?.languages ?? []} />
        </Show>
      </div>
      <aside class={styles.aside}>
        <Budgets targets={statusInfo()?.budgets ?? null} measured={measured(row(), run())} />
        <Recent rows={projects()} />
      </aside>
    </section>
  );
}
