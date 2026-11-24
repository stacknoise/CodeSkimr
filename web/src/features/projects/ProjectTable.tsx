import { For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { TONE } from "../../shared/ui/consts";
import { cx } from "../../shared/ui/cx";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, ResizableTh, RestHead } from "../../shared/table/ResizableHead";
import { Icon } from "../../shared/ui/Icon";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { PROJECT_COLUMNS, PROJECT_TABLE_KEY, RESCAN_ICON, SCAN_MODE, SOURCE_KIND } from "./consts";
import { ago, count, seconds } from "./format";
import { LanguageBar } from "./LanguageBar";
import { Level2Switch } from "./Level2Switch";
import { projectsScanning, renamingProject, scanSeconds, toolsOf } from "./projects";
import { RemoveAction } from "./RemoveAction";
import { RenameButton, RenameField } from "./RenameForm";
import { lastScanLine, level2Summary } from "./reportText";
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
  onRemove: (id: number) => Promise<boolean>;
}

const columns = createTableColumns(
  PROJECT_TABLE_KEY,
  PROJECT_COLUMNS.map((column) => column.id),
);
/** The classes of the head cells that are not plain. */
const HEAD_CLASS: Record<string, string> = {
  languages: styles.languages ?? "",
  types: styles.number ?? "",
  loc: styles.number ?? "",
};

/** The project table (mock 1). Every column can be widened and narrowed at the handle on its right edge. */
export function ProjectTable(props: ProjectTableProps) {
  return (
    <div class={styles.wrap}>
      <table
        class={styles.table}
        style={columns.style()}
        aria-label={t(I18N_KEYS.PROJECTS_LIST_LABEL)}
      >
        <ColumnWidths columns={columns} />
        <thead>
          <tr>
            <For each={PROJECT_COLUMNS}>
              {(column) => (
                <ResizableTh
                  columns={columns}
                  id={column.id}
                  name={t(column.label)}
                  class={HEAD_CLASS[column.id]}
                >
                  {t(column.label)}
                </ResizableTh>
              )}
            </For>
            <RestHead columns={columns} />
          </tr>
        </thead>
        <tbody>
          <For each={props.rows}>{(row) => <Row row={row} table={props} />}</For>
        </tbody>
      </table>
    </div>
  );
}

/** Project name button and its location. */
function NameCell(props: { row: ProjectRow; onOpen: (id: number) => void }) {
  return (
    <td>
      <Show when={renamingProject()?.id !== props.row.id}>
        <button
          type="button"
          class={styles.name}
          aria-label={t(I18N_KEYS.PROJECTS_OPEN, { name: props.row.name })}
          onClick={() => props.onOpen(props.row.id)}
        >
          {props.row.name}
        </button>
      </Show>
      <RenameButton id={props.row.id} name={props.row.name} />
      <RenameField id={props.row.id} name={props.row.name} />
      <div class={styles.path}>{props.row.location}</div>
    </td>
  );
}

/** When and how the project was last scanned. */
function ScanCell(props: { row: ProjectRow }) {
  const scan = () => lastScan(props.row);
  return (
    <td class={styles.muted}>
      {scan().when}
      <div class={styles.mono}>{scan().detail}</div>
      <Show when={props.row.report}>
        {(report) => (
          <>
            <div class={styles.mono}>{lastScanLine(report())}</div>
            <For each={level2Summary(report())}>
              {(line) => <div class={styles.mono}>{line}</div>}
            </For>
          </>
        )}
      </Show>
    </td>
  );
}

/** Status pills, the rescan button and the removal. */
function StatusCell(props: {
  row: ProjectRow;
  onRescan: (id: number) => void;
  onRemove: (id: number) => Promise<boolean>;
}) {
  const state = () => status(props.row, projectsScanning().has(props.row.id));
  const problems = () => problemsOf(toolsOf(props.row)).length;
  return (
    <td>
      <div class={styles.state}>
        <Pill tone={state().tone}>{state().text}</Pill>
        <Show when={problems() > 0}>
          <Pill tone={TONE.warn}>
            {t(I18N_KEYS.PROJECTS_TOOLS_MISSING_COUNT, { count: problems() })}
          </Pill>
        </Show>
        <Show when={!props.row.running}>
          <button
            type="button"
            class={styles.rescan}
            aria-label={t(I18N_KEYS.PROJECTS_RESCAN, { name: props.row.name })}
            onClick={(event) => {
              event.stopPropagation();
              props.onRescan(props.row.id);
            }}
          >
            <Icon name={RESCAN_ICON} />
          </button>
          <RemoveAction
            id={props.row.id}
            name={props.row.name}
            onRemove={() => props.onRemove(props.row.id)}
          />
        </Show>
      </div>
    </td>
  );
}

function Row(props: { row: ProjectRow; table: ProjectTableProps }) {
  return (
    <tr
      class={cx(styles.row, props.table.selected === props.row.id && styles.selected)}
      aria-current={props.table.selected === props.row.id}
      onClick={() => props.table.onSelect(props.row.id)}
      onDblClick={() => props.table.onOpen(props.row.id)}
    >
      <NameCell row={props.row} onOpen={props.table.onOpen} />
      <td class={styles.muted}>{sourceText(props.row)}</td>
      <td>
        <LanguageBar languages={props.row.languages} />
      </td>
      <td class={styles.number}>{count(props.row.types, currentLanguage())}</td>
      <td class={styles.number}>{count(props.row.loc, currentLanguage())}</td>
      <ScanCell row={props.row} />
      <td>
        <Level2Switch row={props.row} />
      </td>
      <StatusCell row={props.row} onRescan={props.table.onRescan} onRemove={props.table.onRemove} />
    </tr>
  );
}
