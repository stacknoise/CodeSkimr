import { createSignal, For, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { latestRun } from "../scan/run";
import { scanRuns } from "../scan/events";
import { percent } from "../scan/run";
import { PERCENT_SIGN, PROGRESS_MAX, PROGRESS_MIN, STATE } from "../scan/consts";
import { PART_SEPARATOR } from "./consts";
import { ago, count, languageShares } from "./format";
import { projectsScanning } from "./projects";
import { removingNow } from "./removing";
import { stateOf } from "./state";
import type { ProjectRow } from "./types";
import styles from "./ProjectList.module.css";

/** The line under the name: the main language and when the project was last scanned. */
function line(row: ProjectRow): string {
  const main = languageShares(row.languages)[0]?.name;
  const when =
    row.scan === null
      ? t(I18N_KEYS.PROJECTS_NEVER)
      : ago(row.scan.created, Date.now(), currentLanguage());
  return [main, when].filter((part) => part !== undefined).join(PART_SEPARATOR);
}

/** The thin bar of a project whose scan runs; it says nothing before the first report. */
function Mini(props: { row: ProjectRow }) {
  const run = () => latestRun(scanRuns(), props.row.id);
  return (
    <Show when={run()?.state === STATE.running ? run() : undefined}>
      {(current) => (
        <div
          class={styles.mini}
          role="progressbar"
          aria-label={t(I18N_KEYS.PROJECTS_LIST_PROGRESS, { name: props.row.name })}
          aria-valuemin={PROGRESS_MIN}
          aria-valuemax={PROGRESS_MAX}
          aria-valuenow={percent(current())}
        >
          <div class={styles.fill} style={{ width: percent(current()) + PERCENT_SIGN }} />
        </div>
      )}
    </Show>
  );
}

/** One project of the list: a dot for the state (with its words for a screen reader), the name and
 * one line. A project that is being removed cannot be chosen. */
function Item(props: { row: ProjectRow; selected: boolean; onSelect: (id: number) => void }) {
  const state = () => stateOf(props.row, projectsScanning().has(props.row.id));
  return (
    <li>
      <button
        type="button"
        class={cx(styles.item, props.selected && styles.selected)}
        aria-current={props.selected}
        aria-label={t(I18N_KEYS.PROJECTS_OPEN, { name: props.row.name })}
        disabled={removingNow(props.row)}
        onClick={() => props.onSelect(props.row.id)}
      >
        <span
          class={cx(styles.dot, styles[state().tone])}
          role="img"
          aria-label={state().text}
          title={state().text}
        />
        <span class={styles.text}>
          <span class={styles.name}>{props.row.name}</span>
          <span class={styles.line}>{removingNow(props.row) ? state().text : line(props.row)}</span>
          <Mini row={props.row} />
        </span>
      </button>
    </li>
  );
}

/** Props of the list. */
export interface ProjectListProps {
  rows: ProjectRow[];
  selected: number | null;
  onSelect: (id: number) => void;
  onNew: () => void;
}

/** The head of the list: the title and the button "New". */
function Head(props: { onNew: () => void }) {
  return (
    <div class={styles.head}>
      <h1>{t(I18N_KEYS.PROJECTS_TITLE)}</h1>
      <button type="button" class={styles.new} onClick={props.onNew}>
        {t(I18N_KEYS.PROJECTS_NEW_SHORT)}
      </button>
    </div>
  );
}

/** The field that narrows the list by name. */
function Filter(props: { value: string; onType: (value: string) => void }) {
  return (
    <input
      class={styles.filter}
      type="search"
      aria-label={t(I18N_KEYS.PROJECTS_FILTER_LABEL)}
      placeholder={t(I18N_KEYS.PROJECTS_FILTER_PLACEHOLDER)}
      value={props.value}
      onInput={(event) => props.onType(event.currentTarget.value)}
      spellcheck={false}
      autocomplete="off"
    />
  );
}

/** The narrow list of the projects screen (mock 23): the button "New", a filter and one entry per
 * project. */
export function ProjectList(props: ProjectListProps) {
  const [filter, setFilter] = createSignal("");
  const shown = () => {
    const needle = filter().trim().toLowerCase();
    return props.rows.filter((row) => row.name.toLowerCase().includes(needle));
  };
  const types = () => props.rows.reduce((sum, row) => sum + row.types, 0);
  return (
    <nav class={styles.list} aria-label={t(I18N_KEYS.PROJECTS_LIST_LABEL)}>
      <Head onNew={props.onNew} />
      <Filter value={filter()} onType={setFilter} />
      <ul class={styles.items}>
        <For each={shown()}>
          {(row) => (
            <Item row={row} selected={props.selected === row.id} onSelect={props.onSelect} />
          )}
        </For>
      </ul>
      <Show when={shown().length === 0}>
        <p class={styles.none}>{t(I18N_KEYS.PROJECTS_FILTER_NONE)}</p>
      </Show>
      <p class={styles.foot}>
        {t(I18N_KEYS.PROJECTS_SUMMARY, {
          projects: props.rows.length,
          types: count(types(), currentLanguage()),
        })}
      </p>
    </nav>
  );
}
