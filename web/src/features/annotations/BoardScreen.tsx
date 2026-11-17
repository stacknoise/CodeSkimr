import { createEffect, createMemo, createSignal, For, Match, Show, Switch } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { cx } from "../../shared/ui/cx";
import { updateAnnotation } from "./api";
import {
  boardNotes,
  boardState,
  boardTasks,
  column,
  filterTasks,
  isMarked,
  loadBoard,
  NO_FILTERS,
  openTasks,
  type Filters,
} from "./board";
import { TaskMeta } from "./Cards";
import {
  BOARD,
  COLUMNS,
  FILTER,
  SHIELD,
  STATUS,
  STATUS_LABEL,
  STATUSES,
  type TaskStatus,
} from "./consts";
import { shortDate } from "./format";
import { showTarget } from "./jump";
import { myName } from "./me";
import { Shield } from "./Marks";
import { annotationChanges, change } from "./store";
import type { Annotation } from "./types";
import styles from "./BoardScreen.module.css";

/** The filter buttons above the columns; each is a switch that says whether it is on. */
function FilterButton(props: {
  label: string;
  on: boolean;
  disabled?: boolean | undefined;
  title?: string | undefined;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      class={cx(styles.filter, props.on && styles.on)}
      aria-pressed={props.on}
      disabled={props.disabled}
      title={props.title}
      onClick={props.onToggle}
    >
      {props.label}
    </button>
  );
}

/** The status of a task as a list, so that a task moves between the columns without a mouse. */
function StatusPick(props: { project: number; task: Annotation }) {
  return (
    <select
      class={styles.status}
      aria-label={t(I18N_KEYS.ANNOTATIONS_TASK_STATUS)}
      value={props.task.status ?? STATUS.open}
      onChange={(event) =>
        void change(props.project, () =>
          updateAnnotation(props.project, props.task.id, { status: event.currentTarget.value }),
        )
      }
    >
      <For each={STATUSES}>
        {(status) => <option value={status}>{t(STATUS_LABEL[status])}</option>}
      </For>
    </select>
  );
}

/** A task as a card (mock 16): the symbol, the title, the due date and the assignee. */
function Card(props: { project: number; task: Annotation }) {
  const marked = () => isMarked(props.task.target);
  return (
    <li class={styles.card}>
      <button
        type="button"
        class={styles.symbol}
        title={t(I18N_KEYS.BOARD_OPEN_SYMBOL, { name: props.task.target.name })}
        onClick={() => void showTarget(props.project, props.task.target)}
      >
        <Show when={marked()}>
          <span
            class={styles.shield}
            role="img"
            aria-label={t(I18N_KEYS.ANNOTATIONS_CRITICAL_LABEL)}
          >
            <Shield size={SHIELD.label} />
          </span>
        </Show>
        <span class={styles.name}>{props.task.target.name}</span>
      </button>
      <p class={styles.task}>{props.task.title}</p>
      <Show when={props.task.status === STATUS.done} fallback={<TaskMeta task={props.task} />}>
        <span class={styles.done}>
          {t(I18N_KEYS.BOARD_DONE_ON, {
            date: shortDate(props.task.updated, currentLanguage()),
          })}
        </span>
      </Show>
      <StatusPick project={props.project} task={props.task} />
    </li>
  );
}

/** One column of the board: the name of the status, the number and the cards. */
function Column(props: { project: number; status: TaskStatus; tasks: Annotation[] }) {
  const cards = () => column(props.tasks, props.status);
  return (
    <section class={styles.column}>
      <h2 class={styles.head}>
        {t(I18N_KEYS.BOARD_COLUMN, { name: t(STATUS_LABEL[props.status]), count: cards().length })}
      </h2>
      <Show
        when={cards().length > 0}
        fallback={<p class={styles.none}>{t(I18N_KEYS.BOARD_EMPTY_COLUMN)}</p>}
      >
        <ul class={styles.cards}>
          <For each={cards()}>{(task) => <Card project={props.project} task={task} />}</For>
        </ul>
      </Show>
    </section>
  );
}

/** The head of the board: the title, the two counts and the filters. */
function Head(props: { filters: Filters; onChange: (next: Filters) => void }) {
  const set = (key: keyof Filters) =>
    props.onChange({ ...props.filters, [key]: !props.filters[key] });
  return (
    <div class={styles.bar}>
      <h1 class={styles.title}>{t(I18N_KEYS.BOARD_TITLE)}</h1>
      <span class={styles.chip}>
        {t(I18N_KEYS.BOARD_CHIP_TASKS, { count: openTasks(boardTasks()) })}
      </span>
      <span class={styles.chip}>{t(I18N_KEYS.BOARD_CHIP_NOTES, { count: boardNotes() })}</span>
      <span class={styles.fill} />
      <div class={styles.filters} role="group" aria-label={t(I18N_KEYS.BOARD_FILTERS_LABEL)}>
        <FilterButton
          label={t(I18N_KEYS.BOARD_FILTER_MINE)}
          on={props.filters.mine}
          disabled={myName() === null}
          title={myName() === null ? t(I18N_KEYS.BOARD_FILTER_MINE_UNKNOWN) : undefined}
          onToggle={() => set(FILTER.mine)}
        />
        <FilterButton
          label={t(I18N_KEYS.BOARD_FILTER_CRITICAL)}
          on={props.filters.critical}
          onToggle={() => set(FILTER.critical)}
        />
        <FilterButton
          label={t(I18N_KEYS.BOARD_FILTER_DUE)}
          on={props.filters.due}
          onToggle={() => set(FILTER.due)}
        />
      </div>
    </div>
  );
}

/** The board of the tasks of the project (mock 16, `Tasks.dc.html`): a column for each status. */
export function BoardScreen() {
  const [filters, setFilters] = createSignal<Filters>(NO_FILTERS);
  createEffect(() => {
    const project = activeProject();
    annotationChanges();
    if (project !== null) {
      void loadBoard(project);
    }
  });
  const shown = createMemo(() => filterTasks(boardTasks(), filters(), isMarked, myName()));
  return (
    <div class={styles.screen}>
      <Head filters={filters()} onChange={setFilters} />
      <Switch>
        <Match when={boardState() === BOARD.failed}>
          <p class={styles.note} role="alert">
            {t(I18N_KEYS.BOARD_FAILED)}
          </p>
        </Match>
        <Match when={boardState() === BOARD.loading}>
          <p class={styles.note}>{t(I18N_KEYS.BOARD_LOADING)}</p>
        </Match>
        <Match when={boardTasks().length === 0}>
          <p class={styles.note}>{t(I18N_KEYS.BOARD_EMPTY)}</p>
        </Match>
        <Match when={activeProject()}>
          {(project) => (
            <div class={styles.columns}>
              <For each={COLUMNS}>
                {(status) => <Column project={project()} status={status} tasks={shown()} />}
              </For>
            </div>
          )}
        </Match>
      </Switch>
    </div>
  );
}
