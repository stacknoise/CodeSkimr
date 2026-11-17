import { createSignal, Show, type JSX } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { deleteAnnotation, updateAnnotation } from "./api";
import { MS_PER_SECOND, STATUS } from "./consts";
import { dueDate, isLate, shortDate } from "./format";
import { NoteForm, TaskForm } from "./Forms";
import { RichText } from "./RichText";
import { change, type ChangeFailure } from "./store";
import type { Annotation, Draft } from "./types";
import styles from "./Cards.module.css";

/** "Delete" of a card: it asks once more before it deletes. Also used by the board. */
export function DeleteAction(props: {
  onDelete: () => Promise<ChangeFailure | null>;
  onFailed: (failure: ChangeFailure) => void;
}) {
  const [asking, setAsking] = createSignal(false);
  const remove = async () => {
    const failure = await props.onDelete();
    setAsking(false);
    if (failure !== null) {
      props.onFailed(failure);
    }
  };
  return (
    <Show
      when={asking()}
      fallback={
        <button type="button" onClick={() => setAsking(true)}>
          {t(I18N_KEYS.ANNOTATIONS_DELETE)}
        </button>
      }
    >
      <button type="button" class={styles.danger} onClick={() => void remove()}>
        {t(I18N_KEYS.ANNOTATIONS_DELETE_CONFIRM)}
      </button>
      <button type="button" onClick={() => setAsking(false)}>
        {t(I18N_KEYS.ANNOTATIONS_CANCEL)}
      </button>
    </Show>
  );
}

/** The buttons of a card in a row: `children` (for example "Edit") and "Delete". */
export function ActionRow(props: { children?: JSX.Element }) {
  return <span class={styles.actions}>{props.children}</span>;
}

/** "Edit" and "Delete" of a card. */
function CardActions(props: {
  onEdit: () => void;
  onDelete: () => Promise<ChangeFailure | null>;
  onFailed: (failure: ChangeFailure) => void;
}) {
  return (
    <ActionRow>
      <button type="button" onClick={props.onEdit}>
        {t(I18N_KEYS.ANNOTATIONS_EDIT)}
      </button>
      <DeleteAction onDelete={props.onDelete} onFailed={props.onFailed} />
    </ActionRow>
  );
}

/** Who wrote it and when. */
function Byline(props: { item: Annotation }) {
  return (
    <span>
      {t(I18N_KEYS.ANNOTATIONS_BY, {
        author: props.item.author,
        date: shortDate(props.item.updated, currentLanguage()),
      })}
    </span>
  );
}

/** Changes an annotation, and removes it, on the server. */
function useEdits(project: number, item: () => Annotation) {
  return {
    update: (draft: Draft) => change(project, () => updateAnnotation(project, item().id, draft)),
    remove: () => change(project, () => deleteAnnotation(project, item().id)),
  };
}

/** A note of the side panel (mock 14): its text, who wrote it and when, and the buttons. */
export function NoteCard(props: {
  project: number;
  note: Annotation;
  onFailed: (failure: ChangeFailure) => void;
}) {
  const [editing, setEditing] = createSignal(false);
  const edits = useEdits(props.project, () => props.note);
  return (
    <Show
      when={!editing()}
      fallback={
        <NoteForm
          initial={props.note.text ?? ""}
          save={(text) => edits.update({ text })}
          onClose={() => setEditing(false)}
        />
      }
    >
      <div class={styles.note}>
        <RichText text={props.note.text ?? ""} />
        <small class={styles.meta}>
          <Byline item={props.note} />
          <CardActions
            onEdit={() => setEditing(true)}
            onDelete={edits.remove}
            onFailed={props.onFailed}
          />
        </small>
      </div>
    </Show>
  );
}

/** The line under the title of a task: the due date, late or not, and the assignee. */
export function TaskMeta(props: { task: Annotation }) {
  const due = () => props.task.due ?? null;
  const late = () => {
    const at = due();
    return (
      at !== null && props.task.status !== STATUS.done && isLate(at, Date.now() / MS_PER_SECOND)
    );
  };
  return (
    <span class={cx(styles.taskMeta, late() && styles.late)}>
      <Show when={due()} fallback={t(I18N_KEYS.ANNOTATIONS_TASK_NO_DUE)}>
        {(at) =>
          t(late() ? I18N_KEYS.ANNOTATIONS_TASK_LATE : I18N_KEYS.ANNOTATIONS_TASK_DUE_LINE, {
            date: dueDate(at(), currentLanguage()),
          })
        }
      </Show>
      <Show when={props.task.assignee}>
        {(name) => (
          <span class={styles.assignee}>
            {t(I18N_KEYS.ANNOTATIONS_TASK_ASSIGNED, { name: name() })}
          </span>
        )}
      </Show>
    </span>
  );
}

/** The box of a task: done or not, as a real checkbox button. */
function Check(props: { task: Annotation; onToggle: () => void }) {
  const done = () => props.task.status === STATUS.done;
  return (
    <button
      type="button"
      role="checkbox"
      class={cx(styles.box, done() && styles.done)}
      aria-checked={done()}
      aria-label={t(I18N_KEYS.ANNOTATIONS_TASK_DONE_TOGGLE, { title: props.task.title ?? "" })}
      onClick={props.onToggle}
    />
  );
}

/** Title, description, due date and buttons of a task. */
function TaskBody(props: {
  task: Annotation;
  onEdit: () => void;
  onDelete: () => Promise<ChangeFailure | null>;
  onFailed: (failure: ChangeFailure) => void;
}) {
  return (
    <div class={styles.taskBody}>
      <span class={cx(styles.title, props.task.status === STATUS.done && styles.struck)}>
        {props.task.title}
      </span>
      <Show when={props.task.text}>
        {(text) => (
          <span class={styles.description}>
            <RichText text={text()} />
          </span>
        )}
      </Show>
      <TaskMeta task={props.task} />
      <CardActions onEdit={props.onEdit} onDelete={props.onDelete} onFailed={props.onFailed} />
    </div>
  );
}

/** A task of the side panel (mock 14): check box, title, due date and assignee. */
export function TaskCard(props: {
  project: number;
  task: Annotation;
  onFailed: (failure: ChangeFailure) => void;
}) {
  const [editing, setEditing] = createSignal(false);
  const edits = useEdits(props.project, () => props.task);
  const toggle = async () => {
    const done = props.task.status === STATUS.done;
    const failure = await edits.update({ status: done ? STATUS.open : STATUS.done });
    if (failure !== null) {
      props.onFailed(failure);
    }
  };
  return (
    <Show
      when={!editing()}
      fallback={
        <TaskForm initial={props.task} save={edits.update} onClose={() => setEditing(false)} />
      }
    >
      <div class={styles.task}>
        <Check task={props.task} onToggle={() => void toggle()} />
        <TaskBody
          task={props.task}
          onEdit={() => setEditing(true)}
          onDelete={edits.remove}
          onFailed={props.onFailed}
        />
      </div>
    </Show>
  );
}
