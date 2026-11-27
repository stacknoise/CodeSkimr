import { createEffect, createSignal, For, on, onCleanup, Show } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { currentLanguage, errorText, t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { uiConfig } from "../../shared/settings/ui-config";
import { createTableColumns } from "../../shared/table/columns";
import { ColumnWidths, RestHead } from "../../shared/table/ResizableHead";
import type { SortState } from "../../shared/table/sort";
import { SortHead } from "../../shared/table/SortHead";
import { ErrorDetails } from "../../shared/ui/ErrorDetails";
import { Pill } from "../../shared/ui/Pill";
import { TONE } from "../../shared/ui/consts";
import { openPlaceInEditor } from "../explorer/editor";
import { showInExplorer } from "../rules/jump";
import { SOURCE_KIND } from "../projects/consts";
import { count } from "../projects/format";
import { projects } from "../projects/projects";
import {
  COLUMN,
  COLUMNS,
  CONVENTIONS_TABLE_KEY,
  DEFAULT_SORT,
  NUMBER_COLUMNS,
  PLACE_SEPARATOR,
  type Column,
} from "./consts";
import {
  conventionsAnswer,
  conventionsCheckedAt,
  conventionsFailure,
  conventionsRunning,
  resetConventions,
  runConventionsCheck,
} from "./conventions";
import { markdownOf } from "./copy";
import { createCopier } from "../../shared/ui/copier";
import { COPY } from "../../shared/ui/consts";
import { measureText, placeText } from "./measure";
import { sortFindings } from "./order";
import type { ConventionsOut, Finding } from "./types";
import styles from "./ConventionsScreen.module.css";

const columns = createTableColumns(CONVENTIONS_TABLE_KEY, COLUMNS);
const [sort, setSort] = createSignal<SortState | null>(DEFAULT_SORT);

/** The project that is shown, as the list of projects knows it. */
const shownProject = () => projects().find((row) => row.id === activeProject());

/** Why the check cannot be started, `null` if it can. */
function blockedReason(): MessageKey | null {
  return shownProject()?.kind === SOURCE_KIND.git ? I18N_KEYS.CONVENTIONS_GIT_HINT : null;
}

/** The head of a column of the table of findings. */
function Head(props: { column: Column; label: MessageKey; tip?: MessageKey }) {
  return (
    <SortHead
      columns={columns}
      id={props.column}
      label={t(props.label)}
      title={props.tip ? t(props.tip) : undefined}
      sort={sort()}
      numeric={NUMBER_COLUMNS.includes(props.column)}
      onSort={setSort}
    />
  );
}

/** The head row of the table of findings. */
function Heads() {
  return (
    <thead>
      <tr>
        <Head column={COLUMN.rule} label={I18N_KEYS.CONVENTIONS_COL_RULE} />
        <Head column={COLUMN.place} label={I18N_KEYS.CONVENTIONS_COL_PLACE} />
        <Head column={COLUMN.symbol} label={I18N_KEYS.CONVENTIONS_COL_SYMBOL} />
        <Head
          column={COLUMN.measure}
          label={I18N_KEYS.CONVENTIONS_COL_MEASURE}
          tip={I18N_KEYS.CONVENTIONS_COL_MEASURE_HINT}
        />
        <RestHead columns={columns} />
      </tr>
    </thead>
  );
}

/** The button that opens the file of a finding in the editor of the settings. It is off while the
 * editor is not known, and says why. */
function OpenButton(props: { project: number; finding: Finding }) {
  const ready = () => uiConfig() !== null;
  return (
    <button
      type="button"
      class={styles.small}
      disabled={!ready()}
      title={t(ready() ? I18N_KEYS.CONVENTIONS_OPEN_HINT : I18N_KEYS.CONVENTIONS_OPEN_UNAVAILABLE)}
      onClick={() => openPlaceInEditor(props.project, props.finding.path, props.finding.line)}
    >
      {t(I18N_KEYS.CONVENTIONS_OPEN)}
    </button>
  );
}

/** The button that shows the function of a finding in the explorer. It is off if the last scan
 * does not know the function, and says why. */
function ExplorerButton(props: { project: number; finding: Finding }) {
  const symbol = () => props.finding.symbol_ref;
  return (
    <button
      type="button"
      class={styles.small}
      disabled={symbol() === null}
      title={t(
        symbol() === null ? I18N_KEYS.RULES_SYMBOL_GONE : I18N_KEYS.CONVENTIONS_EXPLORER_HINT,
      )}
      onClick={() => {
        const found = symbol();
        if (found !== null) {
          void showInExplorer(props.project, props.finding.symbol, found);
        }
      }}
    >
      {t(I18N_KEYS.CONVENTIONS_EXPLORER)}
    </button>
  );
}

function Row(props: { project: number; finding: Finding }) {
  return (
    <tr>
      <td>
        <Pill tone={TONE.bad} compact title={errorText(props.finding.code as ErrorCode)}>
          {props.finding.code}
        </Pill>
        <div class={styles.detail}>{errorText(props.finding.code as ErrorCode)}</div>
      </td>
      <td class={styles.mono}>{placeText(props.finding, PLACE_SEPARATOR)}</td>
      <td class={styles.mono}>{props.finding.symbol}</td>
      <td class={styles.num}>{measureText(props.finding, PLACE_SEPARATOR)}</td>
      <td>
        <OpenButton project={props.project} finding={props.finding} />
        <ExplorerButton project={props.project} finding={props.finding} />
      </td>
    </tr>
  );
}

/** The findings, as a table that can be sorted and whose columns can be widened. */
function Findings(props: { project: number; answer: ConventionsOut }) {
  return (
    <table
      class={styles.table}
      style={columns.style()}
      aria-label={t(I18N_KEYS.CONVENTIONS_TABLE_LABEL)}
    >
      <ColumnWidths columns={columns} />
      <Heads />
      <tbody>
        <For each={sortFindings(props.answer.findings, currentLanguage(), sort())}>
          {(finding) => <Row project={props.project} finding={finding} />}
        </For>
      </tbody>
    </table>
  );
}

/** One number of the strip above the findings, with what it counts. */
function Figure(props: { value: number; label: MessageKey; bad?: boolean }) {
  return (
    <div class={styles.figure}>
      <span class={props.bad && props.value > 0 ? styles.figureBad : styles.figureValue}>
        {count(props.value, currentLanguage())}
      </span>
      <span class={styles.figureLabel}>{t(props.label)}</span>
    </div>
  );
}

/** Files that could not be analysed and rules that were not checked: what a clean result does not
 * say. */
function Gaps(props: { answer: ConventionsOut }) {
  return (
    <>
      <Show when={props.answer.not_checked.length > 0}>
        <p class={styles.note}>
          {t(I18N_KEYS.CONVENTIONS_NOT_CHECKED_TITLE)} {props.answer.not_checked.join(", ")}
        </p>
      </Show>
      <Show when={props.answer.unchecked.length > 0}>
        <details class={styles.note}>
          <summary>{t(I18N_KEYS.CONVENTIONS_UNCHECKED_TITLE)}</summary>
          <For each={props.answer.unchecked}>
            {(file) => (
              <div class={styles.mono}>
                {file.path} {file.code}
              </div>
            )}
          </For>
        </details>
      </Show>
    </>
  );
}

/** The result of the last check. */
function Result(props: { project: number; answer: ConventionsOut }) {
  return (
    <>
      <div class={styles.summary}>
        <Figure value={props.answer.files_checked} label={I18N_KEYS.CONVENTIONS_FIGURE_FILES} />
        <Figure value={props.answer.total} label={I18N_KEYS.CONVENTIONS_FIGURE_FINDINGS} bad />
        <Figure
          value={props.answer.unchecked.length}
          label={I18N_KEYS.CONVENTIONS_FIGURE_UNCHECKED}
        />
      </div>
      <Show
        when={props.answer.total > 0}
        fallback={
          <p class={styles.note} role="status">
            {t(I18N_KEYS.CONVENTIONS_CLEAN, { count: props.answer.files_checked })}
          </p>
        }
      >
        <Findings project={props.project} answer={props.answer} />
        <Show when={props.answer.total > props.answer.findings.length}>
          <p class={styles.note} role="status">
            {t(I18N_KEYS.CONVENTIONS_MORE, {
              hidden: props.answer.total - props.answer.findings.length,
              shown: props.answer.findings.length,
            })}
          </p>
        </Show>
      </Show>
      <Gaps answer={props.answer} />
    </>
  );
}

/** The failure of the check, with the code and its details. */
function Failure() {
  return (
    <Show when={conventionsFailure()}>
      {(failure) => (
        <div class={styles.alert} role="alert">
          {failure().code === null
            ? t(I18N_KEYS.CONVENTIONS_FAILED)
            : errorText(failure().code as ErrorCode)}
          <Show when={failure().code}>
            {(code) => <ErrorDetails code={code()} message={errorText(code())} />}
          </Show>
        </div>
      )}
    </Show>
  );
}

/** When the last check was done. */
function CheckedAt() {
  return (
    <Show when={conventionsCheckedAt()}>
      {(at) => (
        <span class={styles.note}>
          {t(I18N_KEYS.CONVENTIONS_CHECKED_AT, {
            time: at().toLocaleTimeString(currentLanguage()),
          })}
        </span>
      )}
    </Show>
  );
}

/** The button that copies the result as Markdown; it is off before the first result. */
function CopyButton() {
  const { state, copy } = createCopier();
  const label = (): MessageKey =>
    state() === COPY.copied
      ? I18N_KEYS.CONVENTIONS_COPIED
      : state() === COPY.failed
        ? I18N_KEYS.CONVENTIONS_COPY_FAILED
        : I18N_KEYS.CONVENTIONS_COPY;
  const copyResult = () => {
    const answer = conventionsAnswer();
    if (answer !== null) copy(markdownOf(answer, shownProject()?.name ?? ""));
  };
  return (
    <button
      type="button"
      class={styles.secondary}
      disabled={conventionsAnswer() === null}
      title={conventionsAnswer() === null ? t(I18N_KEYS.CONVENTIONS_COPY_NONE) : undefined}
      onClick={copyResult}
    >
      {t(label())}
    </button>
  );
}

/** The button that starts the check, the folder it reads and the button that copies the result. */
function Toolbar(props: { project: number }) {
  const reason = () => blockedReason();
  return (
    <div class={styles.toolbar}>
      <button
        type="button"
        class={styles.primary}
        disabled={conventionsRunning() || reason() !== null}
        title={t(reason() ?? I18N_KEYS.CONVENTIONS_RUN_HINT)}
        onClick={() => void runConventionsCheck(props.project)}
      >
        {t(conventionsRunning() ? I18N_KEYS.CONVENTIONS_RUNNING : I18N_KEYS.CONVENTIONS_RUN)}
      </button>
      <Show when={shownProject()?.kind === SOURCE_KIND.folder ? shownProject() : undefined}>
        {(row) => (
          <span class={styles.folder}>
            {t(I18N_KEYS.CONVENTIONS_FOLDER, { folder: row().location })}
          </span>
        )}
      </Show>
      <span class={styles.spacer} />
      <CheckedAt />
      <CopyButton />
    </div>
  );
}

/** The screen "Conventions": the convention check of the console for the project
 * folder (ADR 0116). */
export function ConventionsScreen() {
  createEffect(on(activeProject, () => resetConventions()));
  onCleanup(resetConventions);
  return (
    <>
      <header>
        <h1>{t(I18N_KEYS.CONVENTIONS_TITLE)}</h1>
        <p class={styles.sub}>{t(I18N_KEYS.CONVENTIONS_SUBTITLE)}</p>
      </header>
      <Show
        when={activeProject()}
        fallback={<p class={styles.note}>{t(I18N_KEYS.CONVENTIONS_NO_PROJECT)}</p>}
      >
        {(project) => (
          <>
            <Toolbar project={project()} />
            <Failure />
            <Show
              when={conventionsAnswer()}
              fallback={
                <Show when={!conventionsRunning() && conventionsFailure() === null}>
                  <p class={styles.note}>{t(I18N_KEYS.CONVENTIONS_BEFORE_FIRST)}</p>
                </Show>
              }
            >
              {(answer) => <Result project={project()} answer={answer()} />}
            </Show>
          </>
        )}
      </Show>
    </>
  );
}
