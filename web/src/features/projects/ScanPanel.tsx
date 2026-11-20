import { createEffect, For, on, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { ORIGIN_TEXT, OUTCOME } from "../../shared/project/report";
import { cancelScan } from "../scan/cancel";
import {
  LEVEL2_START,
  LEVEL2_STAGE,
  MS_PER_SECOND,
  NOT_STARTED_TEXT,
  SCAN_PANEL_ID_PREFIX,
} from "./consts";
import { clock, languageName } from "./format";
import { focusSwitch, focusWouldBeLostIn } from "./level2";
import { level2DoneOf, level2RunOf, level2StartOf } from "./level2run";
import { projectsScanning, rescanProject } from "./projects";
import { languageLine } from "./reportText";
import type { ProjectRow } from "./types";
import styles from "./Level2Panel.module.css";

/** Puts the focus on the switch of the project when it would be lost, because the button that had
 * it went away. A person who is typing elsewhere keeps the focus. */
function keepFocus(project: number): void {
  if (focusWouldBeLostIn(`${SCAN_PANEL_ID_PREFIX}${project}`)) {
    focusSwitch(project);
  }
}

/** The title of the running pass; the region exists before the pass starts, so that a screen
 * reader announces the start and the change of the phase. */
function RunningTitle(props: { row: ProjectRow }) {
  const run = () => level2RunOf(props.row.id);
  return (
    <div aria-live="polite" class={styles.live}>
      <Show when={run()}>
        {(current) => (
          <strong>
            {t(
              current().stage === LEVEL2_STAGE.importing
                ? I18N_KEYS.PROJECTS_LEVEL2_RUNNING_IMPORTING
                : I18N_KEYS.PROJECTS_LEVEL2_RUNNING,
              { language: languageName(current().language) },
            )}
          </strong>
        )}
      </Show>
    </div>
  );
}

/** "Stage 2 running (C#)" with the way to stop it; stage 1 stays as it is. */
function Running(props: { row: ProjectRow }) {
  const run = () => level2RunOf(props.row.id);
  createEffect(
    on(
      () => run() !== null,
      (running, was) => {
        if (was && !running) {
          keepFocus(props.row.id);
        }
      },
    ),
  );
  return (
    <div class={run() === null ? undefined : styles.notice}>
      <RunningTitle row={props.row} />
      <Show when={run()}>
        <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_BUILD_NOTE)}</p>
        <div class={styles.buttons}>
          <button type="button" class={styles.button} onClick={() => void cancelScan(props.row.id)}>
            {t(I18N_KEYS.PROJECTS_LEVEL2_CANCEL)}
          </button>
          <span class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_CANCEL_NOTE)}</span>
        </div>
      </Show>
    </div>
  );
}

/** Why stage 2 did not start after the last scan from this tab, or how its run ended. */
function Messages(props: { row: ProjectRow }) {
  const start = () => level2StartOf(props.row.id);
  const done = () => level2DoneOf(props.row.id);
  const reason = () => {
    const current = start();
    return current?.state === LEVEL2_START.notStarted
      ? NOT_STARTED_TEXT[current.reason ?? ""]
      : null;
  };
  return (
    <div aria-live="polite" class={styles.live}>
      <Show when={reason()}>{(key) => <p class={styles.text}>{t(key())}</p>}</Show>
      <Show when={done()}>
        {(counts) => <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_FINISHED, counts())}</p>}
      </Show>
    </div>
  );
}

/** What stage 2 did in the last run that stored a report, one line per language. */
function Report(props: { row: ProjectRow }) {
  const second = () => props.row.report?.level2 ?? null;
  const failed = () => second()?.languages.some((line) => line.outcome === OUTCOME.failed) ?? false;
  return (
    <Show
      when={second()}
      fallback={
        <Show when={props.row.report}>
          <p class={styles.text}>{t(I18N_KEYS.PROJECTS_REPORT_NONE)}</p>
        </Show>
      }
    >
      {(report) => (
        <div>
          <p class={styles.text}>
            {t(I18N_KEYS.PROJECTS_REPORT_TITLE, {
              time: clock(report().at * MS_PER_SECOND, currentLanguage()),
              origin: t(ORIGIN_TEXT[report().origin]),
            })}
          </p>
          <ul class={styles.text}>
            <For each={report().languages}>{(line) => <li>{languageLine(line)}</li>}</For>
          </ul>
          <Show when={failed()}>
            <p class={styles.text}>{t(I18N_KEYS.PROJECTS_REPORT_FAILED_NOTE)}</p>
          </Show>
        </div>
      )}
    </Show>
  );
}

/** The scan buttons: a full scan that runs stage 2 where it is allowed, and a quick one without. */
function Buttons(props: { row: ProjectRow }) {
  const busy = () => props.row.running || projectsScanning().has(props.row.id);
  return (
    <Show when={!busy()}>
      <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_BUILD_NOTE)}</p>
      <div class={styles.buttons}>
        <button
          type="button"
          class={styles.button}
          onClick={() => {
            void rescanProject(props.row.id);
            keepFocus(props.row.id);
          }}
        >
          {t(I18N_KEYS.PROJECTS_LEVEL2_RESCAN)}
        </button>
        <button
          type="button"
          class={styles.button}
          title={t(I18N_KEYS.PROJECTS_LEVEL2_STAGE1_ONLY_HINT)}
          onClick={() => {
            void rescanProject(props.row.id, false);
            keepFocus(props.row.id);
          }}
        >
          {t(I18N_KEYS.PROJECTS_LEVEL2_STAGE1_ONLY)}
        </button>
      </div>
    </Show>
  );
}

/** The scan section of the selected project: progress and cancel of stage 2, why it did not start,
 * the report of the last run and the scan buttons (ADR 0072, decision 10). */
export function ScanPanel(props: { row: ProjectRow }) {
  return (
    <section
      id={`${SCAN_PANEL_ID_PREFIX}${props.row.id}`}
      class={styles.panel}
      aria-label={t(I18N_KEYS.PROJECTS_LEVEL2_SCAN_TITLE)}
    >
      <header class={styles.header}>
        <h2>{t(I18N_KEYS.PROJECTS_LEVEL2_SCAN_TITLE)}</h2>
      </header>
      <Running row={props.row} />
      <Messages row={props.row} />
      <Report row={props.row} />
      <Buttons row={props.row} />
    </section>
  );
}
