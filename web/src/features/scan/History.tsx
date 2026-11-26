import { createMemo, createResource, For, Show } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, ResizableTh, RestHead } from "../../shared/table/ResizableHead";
import { SCAN_MODE } from "../projects/consts";
import { ago, count, seconds } from "../projects/format";
import type { ProjectRow } from "../projects/types";
import {
  COMMAND_SCANS_LIST,
  HISTORY_COLUMNS,
  HISTORY_MOMENT,
  HISTORY_TABLE_KEY,
  MS_PER_SECOND,
  NOTE_SEPARATOR,
  SECONDS_UNIT,
} from "./consts";
import type { ScanEntry, ScanHistoryAnswer } from "./types";
import styles from "./History.module.css";

const columns = createTableColumns(
  HISTORY_TABLE_KEY,
  HISTORY_COLUMNS.map((column) => column.id),
);

/** The day and the time of a scan, in the language of the screen. */
function moment(created: number): string {
  return new Date(created * MS_PER_SECOND).toLocaleString(currentLanguage(), HISTORY_MOMENT);
}

/** A duration in milliseconds as "2,4 s", or a dash for a scan stored before it was kept. */
function took(ms: number | null): string {
  return ms === null
    ? t(I18N_KEYS.SCAN_UNAVAILABLE)
    : `${seconds(ms / MS_PER_SECOND, currentLanguage())}${SECONDS_UNIT}`;
}

function modeText(mode: ScanEntry["mode"]): string {
  return t(
    mode === SCAN_MODE.incremental ? I18N_KEYS.SCAN_MODE_INCREMENTAL : I18N_KEYS.SCAN_MODE_FULL,
  );
}

/** One figure above the table. */
function Card(props: { label: string; value: string; note: string }) {
  return (
    <div class={styles.card}>
      <span class={styles.cardLabel}>{props.label}</span>
      <strong class={styles.cardValue}>{props.value}</strong>
      <span class={styles.cardNote}>{props.note}</span>
    </div>
  );
}

/** The cards over the list: the newest scan, its files and the average of the list. */
function Cards(props: { answer: ScanHistoryAnswer }) {
  const newest = () => props.answer.scans[0];
  const lang = () => currentLanguage();
  return (
    <Show when={newest()}>
      {(scan) => (
        <div class={styles.cards}>
          <Card
            label={t(I18N_KEYS.SCAN_HISTORY_LAST)}
            value={took(scan().duration_ms)}
            note={[ago(scan().created, Date.now(), lang()), modeText(scan().mode)].join(
              NOTE_SEPARATOR,
            )}
          />
          <Card
            label={t(I18N_KEYS.SCAN_HISTORY_COLUMN_FILES)}
            value={count(scan().files, lang())}
            note={
              scan().files_changed === null
                ? t(I18N_KEYS.SCAN_UNAVAILABLE)
                : t(I18N_KEYS.SCAN_HISTORY_FILES_CHANGED, { count: scan().files_changed ?? 0 })
            }
          />
          <Card
            label={t(I18N_KEYS.SCAN_HISTORY_AVERAGE)}
            value={took(props.answer.average_ms)}
            note={t(I18N_KEYS.SCAN_HISTORY_AVERAGE_HINT, { count: props.answer.scans.length })}
          />
        </div>
      )}
    </Show>
  );
}

function HistoryHead() {
  return (
    <thead>
      <tr>
        <For each={HISTORY_COLUMNS}>
          {(column) => (
            <ResizableTh
              columns={columns}
              id={column.id}
              name={t(column.label)}
              class={column.number ? styles.number : undefined}
            >
              {t(column.label)}
            </ResizableTh>
          )}
        </For>
        <RestHead columns={columns} />
      </tr>
    </thead>
  );
}

/** One row of the table: the newest scan carries the mark "current". */
function Row(props: { scan: ScanEntry; newest: boolean }) {
  const lang = () => currentLanguage();
  return (
    <tr class={props.newest ? styles.current : undefined}>
      <td>
        {moment(props.scan.created)}
        <Show when={props.newest}>
          <span class={styles.badge}>{t(I18N_KEYS.SCAN_HISTORY_CURRENT)}</span>
        </Show>
      </td>
      <td>{modeText(props.scan.mode)}</td>
      <td class={styles.number}>{took(props.scan.duration_ms)}</td>
      <td class={styles.number}>{count(props.scan.files, lang())}</td>
      <td class={styles.number}>
        {props.scan.files_changed === null
          ? t(I18N_KEYS.SCAN_UNAVAILABLE)
          : count(props.scan.files_changed, lang())}
      </td>
    </tr>
  );
}

/** The table of the scans and the note under it. */
function Table(props: { scans: readonly ScanEntry[] }) {
  return (
    <>
      <div class={styles.wrap}>
        <table class={styles.table} style={columns.style()}>
          <ColumnWidths columns={columns} />
          <HistoryHead />
          <tbody>
            <For each={props.scans}>
              {(scan, index) => <Row scan={scan} newest={index() === 0} />}
            </For>
          </tbody>
        </table>
      </div>
      <div class={styles.note}>{t(I18N_KEYS.SCAN_HISTORY_NOTE)}</div>
    </>
  );
}

/** The title of the list and how many scans the project keeps. */
function Title(props: { keep: number }) {
  return (
    <div class={styles.title}>
      <h3>{t(I18N_KEYS.SCAN_RECENT_TITLE)}</h3>
      <span class={styles.keep}>{t(I18N_KEYS.SCAN_HISTORY_KEEP, { count: props.keep })}</span>
    </div>
  );
}

/** The last scans of the project, newest first, with the duration of each and the files it
 * changed (ADR 0097, mock 28 a). The list is read again when the newest scan changes. */
export function History(props: { row: ProjectRow }) {
  const source = createMemo(() => ({
    project: props.row.id,
    snapshot: props.row.scan?.snapshot ?? null,
    created: props.row.scan?.created ?? null,
    running: props.row.running,
  }));
  const [answer] = createResource(source, (input) =>
    runCommand<ScanHistoryAnswer>(COMMAND_SCANS_LIST, { project: input.project }),
  );
  return (
    <section class={styles.history} aria-label={t(I18N_KEYS.SCAN_RECENT_TITLE)}>
      <Show when={answer.error}>
        <div class={styles.note} role="status">
          {t(I18N_KEYS.SCAN_HISTORY_FAILED)}
        </div>
      </Show>
      <Show when={answer.error ? undefined : answer.latest}>
        {(list) => (
          <>
            <Cards answer={list()} />
            <Title keep={list().keep} />
            <Show
              when={list().scans.length > 0}
              fallback={<div class={styles.note}>{t(I18N_KEYS.SCAN_HISTORY_EMPTY)}</div>}
            >
              <Table scans={list().scans} />
            </Show>
          </>
        )}
      </Show>
    </section>
  );
}
