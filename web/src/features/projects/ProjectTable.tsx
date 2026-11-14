import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { Icon } from "../../shared/ui/Icon";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { RESCAN_ICON, SCAN_MODE, SOURCE_KIND } from "./consts";
import { ago, count, seconds } from "./format";
import { LanguageBar } from "./LanguageBar";
import { projectsScanning, scanSeconds, toolsOf } from "./projects";
import { problemsOf } from "./tools";
import type { ProjectRow } from "./types";
import styles from "./ProjectTable.module.css";

/** What the status column says about a project. */
function status(row: ProjectRow, scanning: boolean): { text: string; tone: PillTone } {
  if (row.running || scanning) {
    return { text: t(I18N_KEYS.PROJECTS_STATUS_RUNNING), tone: TONE.accent };
  }
  return row.scan === null
    ? { text: t(I18N_KEYS.PROJECTS_STATUS_UNSCANNED), tone: TONE.neutral }
    : { text: t(I18N_KEYS.PROJECTS_STATUS_CURRENT), tone: TONE.good };
}

function sourceText(row: ProjectRow): string {
  return row.kind === SOURCE_KIND.git
    ? t(I18N_KEYS.PROJECTS_SOURCE_GIT)
    : t(I18N_KEYS.PROJECTS_SOURCE_FOLDER);
}

function lastScan(row: ProjectRow): { when: string; detail: string } {
  if (row.scan === null) {
    return { when: t(I18N_KEYS.PROJECTS_NEVER), detail: "" };
  }
  const mode = t(
    row.scan.mode === SCAN_MODE.incremental
      ? I18N_KEYS.PROJECTS_SCAN_INCREMENTAL
      : I18N_KEYS.PROJECTS_SCAN_FULL,
  );
  const took = scanSeconds().get(row.id);
  const detail =
    took === undefined
      ? mode
      : t(I18N_KEYS.PROJECTS_SCAN_TIME, {
          mode,
          seconds: t(I18N_KEYS.PROJECTS_SCAN_SECONDS, {
            seconds: seconds(took, currentLanguage()),
          }),
        });
  return { when: ago(row.scan.created, Date.now(), currentLanguage()), detail };
}

/** Props of the table: which row is selected and what a click does. */
export interface ProjectTableProps {
  rows: ProjectRow[];
  selected: number | null;
  onSelect: (id: number) => void;
  onOpen: (id: number) => void;
  onRescan: (id: number) => void;
}

/** The project table (mock 1). */
export function ProjectTable(props: ProjectTableProps) {
  return (
    <div class={styles.wrap}>
      <table class={styles.table} aria-label={t(I18N_KEYS.PROJECTS_LIST_LABEL)}>
        <thead>
          <tr>
            <th>{t(I18N_KEYS.PROJECTS_COLUMN_PROJECT)}</th>
            <th>{t(I18N_KEYS.PROJECTS_COLUMN_SOURCE)}</th>
            <th class={styles.languages}>{t(I18N_KEYS.PROJECTS_COLUMN_LANGUAGES)}</th>
            <th class={styles.number}>{t(I18N_KEYS.PROJECTS_COLUMN_TYPES)}</th>
            <th class={styles.number}>{t(I18N_KEYS.PROJECTS_COLUMN_LOC)}</th>
            <th>{t(I18N_KEYS.PROJECTS_COLUMN_LAST_SCAN)}</th>
            <th>{t(I18N_KEYS.PROJECTS_COLUMN_STATUS)}</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.rows}>{(row) => <Row row={row} table={props} />}</For>
        </tbody>
      </table>
    </div>
  );
}

function Row(props: { row: ProjectRow; table: ProjectTableProps }) {
  const state = () => status(props.row, projectsScanning().has(props.row.id));
  const scan = () => lastScan(props.row);
  return (
    <tr
      class={cx(styles.row, props.table.selected === props.row.id && styles.selected)}
      aria-current={props.table.selected === props.row.id}
      onClick={() => props.table.onSelect(props.row.id)}
      onDblClick={() => props.table.onOpen(props.row.id)}
    >
      <td>
        <button
          type="button"
          class={styles.name}
          aria-label={t(I18N_KEYS.PROJECTS_OPEN, { name: props.row.name })}
          onClick={() => props.table.onOpen(props.row.id)}
        >
          {props.row.name}
        </button>
        <div class={styles.path}>{props.row.location}</div>
      </td>
      <td class={styles.muted}>{sourceText(props.row)}</td>
      <td>
        <LanguageBar languages={props.row.languages} />
      </td>
      <td class={styles.number}>{count(props.row.types, currentLanguage())}</td>
      <td class={styles.number}>{count(props.row.loc, currentLanguage())}</td>
      <td class={styles.muted}>
        {scan().when}
        <div class={styles.mono}>{scan().detail}</div>
      </td>
      <td>
        <div class={styles.state}>
          <Pill tone={state().tone}>{state().text}</Pill>
          <Show when={problemsOf(toolsOf(props.row)).length > 0}>
            <Pill tone={TONE.warn}>
              {t(I18N_KEYS.PROJECTS_TOOLS_MISSING_COUNT, {
                count: problemsOf(toolsOf(props.row)).length,
              })}
            </Pill>
          </Show>
          <Show when={!props.row.running}>
            <button
              type="button"
              class={styles.rescan}
              aria-label={t(I18N_KEYS.PROJECTS_RESCAN, { name: props.row.name })}
              onClick={(event) => {
                event.stopPropagation();
                props.table.onRescan(props.row.id);
              }}
            >
              <Icon name={RESCAN_ICON} />
            </button>
          </Show>
        </div>
      </td>
    </tr>
  );
}
