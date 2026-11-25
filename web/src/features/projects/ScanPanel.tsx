import { createEffect, For, on, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { ORIGIN_TEXT, OUTCOME, type LanguageReport } from "../../shared/project/report";
import { cx } from "../../shared/ui/cx";
import { cancelScan } from "../scan/cancel";
import {
  LEVEL2_START,
  PROJECT_TAB,
  LEVEL2_STAGE,
  MS_PER_SECOND,
  NOT_STARTED_TEXT,
  SCAN_PANEL_ID_PREFIX,
} from "./consts";
import { clock, languageName } from "./format";
import { focusSwitch, focusWouldBeLostIn } from "./level2";
import { level2DoneOf, level2RunOf, level2StartOf } from "./level2run";
import { showTab } from "./page";
import { projectsScanning, rescanProject } from "./projects";
import { checkedChip, REPORT_ACTION, reportRow, type ReportRow } from "./reportText";
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
export function ScanMessages(props: { row: ProjectRow }) {
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

/** The button next to a language that failed or was skipped. */
function RowAction(props: { row: ProjectRow; action: ReportRow["action"]; language: string }) {
  const busy = () => props.row.running || projectsScanning().has(props.row.id);
  return props.action === REPORT_ACTION.tools ? (
    <button type="button" class={styles.link} onClick={() => showTab(PROJECT_TAB.tools)}>
      {t(I18N_KEYS.PROJECTS_REPORT_ACTION_TOOLS)}
    </button>
  ) : (
    <button
      type="button"
      class={styles.link}
      disabled={busy()}
      onClick={() => void rescanProject(props.row.id, true, false, props.language)}
    >
      {t(I18N_KEYS.PROJECTS_REPORT_ACTION_RESCAN)}
    </button>
  );
}

/** The row of one language that failed or was skipped: what happened, what to do, the button. */
function ReportLine(props: { row: ProjectRow; line: LanguageReport }) {
  const entry = () => reportRow(props.line);
  return (
    <li class={styles.reportRow}>
      <strong>{entry().language}</strong>
      <div class={styles.reason}>
        <p>
          {entry().text}
          <Show when={entry().code}>{(code) => <code class={styles.codeTag}> {code()}</code>}</Show>
        </p>
        <p class={styles.todo}>{entry().todo}</p>
      </div>
      <RowAction row={props.row} action={entry().action} language={props.line.language} />
    </li>
  );
}

/** One group of the report: failed or skipped languages, one row each with what to do. */
function Group(props: {
  row: ProjectRow;
  kind: typeof OUTCOME.failed | typeof OUTCOME.skipped;
  lines: LanguageReport[];
}) {
  const failed = () => props.kind === OUTCOME.failed;
  const title = failed()
    ? I18N_KEYS.PROJECTS_REPORT_GROUP_FAILED
    : I18N_KEYS.PROJECTS_REPORT_GROUP_SKIPPED;
  const count = failed()
    ? I18N_KEYS.PROJECTS_REPORT_GROUP_FAILED_COUNT
    : I18N_KEYS.PROJECTS_REPORT_GROUP_SKIPPED_COUNT;
  return (
    <Show when={props.lines.length > 0}>
      <section class={styles.group} data-group={props.kind}>
        <h3 class={styles.groupTitle}>
          <span class={failed() ? styles.badgeFailed : styles.badgeSkipped}>{t(title)}</span>
          {t(count, { count: props.lines.length })}
        </h3>
        <ul class={cx(styles.rows, failed() ? styles.rowsFailed : styles.rowsSkipped)}>
          <For each={props.lines}>{(line) => <ReportLine row={props.row} line={line} />}</For>
        </ul>
      </section>
    </Show>
  );
}

/** The languages that were checked, as chips with their number of files. */
function CheckedGroup(props: { lines: LanguageReport[] }) {
  return (
    <Show when={props.lines.length > 0}>
      <section class={styles.group} data-group={OUTCOME.checked}>
        <h3 class={styles.groupTitle}>
          <span class={styles.badgeChecked}>{t(I18N_KEYS.PROJECTS_REPORT_GROUP_CHECKED)}</span>
          {t(I18N_KEYS.PROJECTS_REPORT_GROUP_CHECKED_COUNT, { count: props.lines.length })}
        </h3>
        <ul class={cx(styles.rows, styles.rowsChecked, styles.chips)}>
          <For each={props.lines}>{(line) => <li class={styles.chip}>{checkedChip(line)}</li>}</For>
        </ul>
      </section>
    </Show>
  );
}

/** What stage 2 did in the last run that stored a report, grouped by outcome: failed first, then
 * skipped, then checked (mock 25, variant B). */
function Report(props: { row: ProjectRow }) {
  const second = () => props.row.report?.level2 ?? null;
  const lines = (outcome: LanguageReport["outcome"]) =>
    second()?.languages.filter((line) => line.outcome === outcome) ?? [];
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
        <div class={styles.report}>
          <p class={styles.text}>
            {t(I18N_KEYS.PROJECTS_REPORT_TITLE, {
              time: clock(report().at * MS_PER_SECOND, currentLanguage()),
              origin: t(ORIGIN_TEXT[report().origin]),
            })}
          </p>
          <Group row={props.row} kind={OUTCOME.failed} lines={lines(OUTCOME.failed)} />
          <Group row={props.row} kind={OUTCOME.skipped} lines={lines(OUTCOME.skipped)} />
          <CheckedGroup lines={lines(OUTCOME.checked)} />
        </div>
      )}
    </Show>
  );
}

/** One scan button; while a scan runs it stays and is disabled, so the focus is not lost. */
function ScanButton(props: { label: string; hint?: string; disabled: boolean; scan: () => void }) {
  return (
    <button
      type="button"
      class={styles.button}
      title={props.hint}
      disabled={props.disabled}
      onClick={props.scan}
    >
      {props.label}
    </button>
  );
}

/** The scan buttons: a scan of the changes that runs stage 2 where it is allowed, a quick one
 * without, and a full one that reads every file again. */
export function ScanButtons(props: { row: ProjectRow }) {
  const busy = () => props.row.running || projectsScanning().has(props.row.id);
  const id = () => props.row.id;
  return (
    <>
      <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_BUILD_NOTE)}</p>
      <div class={styles.buttons}>
        <ScanButton
          label={t(I18N_KEYS.PROJECTS_LEVEL2_RESCAN)}
          disabled={busy()}
          scan={() => void rescanProject(id())}
        />
        <ScanButton
          label={t(I18N_KEYS.PROJECTS_LEVEL2_STAGE1_ONLY)}
          hint={t(I18N_KEYS.PROJECTS_LEVEL2_STAGE1_ONLY_HINT)}
          disabled={busy()}
          scan={() => void rescanProject(id(), false)}
        />
        <ScanButton
          label={t(I18N_KEYS.PROJECTS_LEVEL2_FULL_RESCAN)}
          hint={t(I18N_KEYS.PROJECTS_LEVEL2_FULL_RESCAN_HINT)}
          disabled={busy()}
          scan={() => void rescanProject(id(), true, true)}
        />
      </div>
    </>
  );
}

/** The scan section of the tab "Exact check": progress and cancel of stage 2, why it did not
 * start and the report of the last run (ADR 0072, decision 10). */
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
      <Report row={props.row} />
    </section>
  );
}
