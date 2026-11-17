import { createEffect, createMemo, createSignal, For, Match, Show, Switch } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject } from "../../shared/project/active";
import { cx } from "../../shared/ui/cx";
import { deleteAnnotation, updateAnnotation } from "./api";
import {
  boardNoteList,
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
import { ActionRow, DeleteAction, TaskMeta } from "./Cards";
import { RichText } from "./RichText";
import {
  BOARD,
  COLUMNS,
  FILTER,
  VIEW,
  type BoardView,
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

/** "Delete" of a card on the board: it asks once more, then removes the annotation. */
function DeleteButton(props: { project: number; item: Annotation }) {
  return (
    <div class={styles.actions}>
      <ActionRow>
        <DeleteAction
          onDelete={() =>
            change(props.project, () => deleteAnnotation(props.project, props.item.id))
          }
          onFailed={() => undefined}
        />
      </ActionRow>
    </div>
  );
}

/** The symbol of a card as a button that shows it in the explorer. */
function SymbolButton(props: { project: number; item: Annotation }) {
  return (
    <button
      type="button"
      class={styles.symbol}
      title={t(I18N_KEYS.BOARD_OPEN_SYMBOL, { name: props.item.target.name })}
      onClick={() => void showTarget(props.project, props.item.target)}
    >
      <Show when={isMarked(props.item.target)}>
        <span class={styles.shield} role="img" aria-label={t(I18N_KEYS.ANNOTATIONS_CRITICAL_LABEL)}>
          <Shield size={SHIELD.label} />
        </span>
      </Show>
      <span class={styles.name}>{props.item.target.name}</span>
    </button>
  );
}

/** A note as a card: the symbol, the text, who wrote it and when. */
function NoteCard(props: { project: number; note: Annotation }) {
  return (
    <li class={styles.card}>
      <SymbolButton project={props.project} item={props.note} />
      <p class={styles.task}>
        <RichText text={props.note.text ?? ""} />
      </p>
      <span class={styles.done}>
        {t(I18N_KEYS.ANNOTATIONS_BY, {
          author: props.note.author,
          date: shortDate(props.note.updated, currentLanguage()),
        })}
      </span>
      <DeleteButton project={props.project} item={props.note} />
    </li>
  );
}

/** The notes of the project as a list of cards. */
function NoteList(props: { project: number }) {
  return (
    <ul class={cx(styles.cards, styles.notes)}>
      <For each={boardNoteList()}>{(note) => <NoteCard project={props.project} note={note} />}</For>
    </ul>
  );
}

/** A task as a card (mock 16): the symbol, the title, the due date and the assignee. */
function Card(props: { project: number; task: Annotation }) {
  return (
    <li class={styles.card}>
      <SymbolButton project={props.project} item={props.task} />
      <p class={styles.task}>{props.task.title}</p>
      <Show when={props.task.status === STATUS.done} fallback={<TaskMeta task={props.task} />}>
        <span class={styles.done}>
          {t(I18N_KEYS.BOARD_DONE_ON, {
            date: shortDate(props.task.updated, currentLanguage()),
          })}
        </span>
      </Show>
      <StatusPick project={props.project} task={props.task} />
      <DeleteButton project={props.project} item={props.task} />
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

/** A switch of the head that shows tasks or notes, with the number of what it shows. */
function ViewButton(props: { label: string; on: boolean; onPick: () => void }) {
  return (
    <button
      type="button"
      class={cx(styles.view, props.on && styles.on)}
      aria-pressed={props.on}
      onClick={props.onPick}
    >
      {props.label}
    </button>
  );
}

/** The filters of the tasks: they make no sense for notes. */
function TaskFilters(props: { filters: Filters; onChange: (next: Filters) => void }) {
  const set = (key: keyof Filters) =>
    props.onChange({ ...props.filters, [key]: !props.filters[key] });
  return (
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
  );
}

/** The head of the board: the title, the switch between tasks and notes, and the filters. */
function Head(props: {
  filters: Filters;
  onChange: (next: Filters) => void;
  view: BoardView;
  onView: (view: BoardView) => void;
}) {
  return (
    <div class={styles.bar}>
      <h1 class={styles.title}>{t(I18N_KEYS.BOARD_TITLE)}</h1>
      <div class={styles.views} role="group" aria-label={t(I18N_KEYS.BOARD_VIEW_LABEL)}>
        <ViewButton
          label={t(I18N_KEYS.BOARD_CHIP_TASKS, { count: openTasks(boardTasks()) })}
          on={props.view === VIEW.tasks}
          onPick={() => props.onView(VIEW.tasks)}
        />
        <ViewButton
          label={t(I18N_KEYS.BOARD_CHIP_NOTES, { count: boardNoteList().length })}
          on={props.view === VIEW.notes}
          onPick={() => props.onView(VIEW.notes)}
        />
      </div>
      <span class={styles.fill} />
      <Show when={props.view === VIEW.tasks}>
        <TaskFilters filters={props.filters} onChange={props.onChange} />
      </Show>
    </div>
  );
}

/** What the board shows once it has read: the columns of the tasks, or the list of the notes. */
function Content(props: { project: number; view: BoardView; tasks: Annotation[] }) {
  return (
    <Switch>
      <Match when={props.view === VIEW.notes && boardNoteList().length === 0}>
        <p class={styles.note}>{t(I18N_KEYS.BOARD_EMPTY_NOTES)}</p>
      </Match>
      <Match when={props.view === VIEW.notes}>
        <NoteList project={props.project} />
      </Match>
      <Match when={boardTasks().length === 0}>
        <p class={styles.note}>{t(I18N_KEYS.BOARD_EMPTY)}</p>
      </Match>
      <Match when={true}>
        <div class={styles.columns}>
          <For each={COLUMNS}>
            {(status) => <Column project={props.project} status={status} tasks={props.tasks} />}
          </For>
        </div>
      </Match>
    </Switch>
  );
}

/** The board of the project (mock 16, `Tasks.dc.html`): the tasks with a column for each status, or
 * the notes, as the switch in the head says. */
export function BoardScreen() {
  const [filters, setFilters] = createSignal<Filters>(NO_FILTERS);
  const [view, setView] = createSignal<BoardView>(VIEW.tasks);
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
      <Head filters={filters()} onChange={setFilters} view={view()} onView={setView} />
      <Switch>
        <Match when={boardState() === BOARD.failed}>
          <p class={styles.note} role="alert">
            {t(I18N_KEYS.BOARD_FAILED)}
          </p>
        </Match>
        <Match when={boardState() === BOARD.loading}>
          <p class={styles.note}>{t(I18N_KEYS.BOARD_LOADING)}</p>
        </Match>
        <Match when={activeProject()}>
          {(project) => <Content project={project()} view={view()} tasks={shown()} />}
        </Match>
      </Switch>
    </div>
  );
}
