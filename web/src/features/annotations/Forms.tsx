import { createSignal, For, onMount, Show, type JSX } from "solid-js";
import { t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { STATUS_LABEL, STATUSES, STATUS, TEXT_ROWS, type TaskStatus } from "./consts";
import { dateFieldOf, dueOfField } from "./format";
import { failureText } from "./panel";
import type { ChangeFailure } from "./store";
import type { Annotation, Draft } from "./types";
import styles from "./Forms.module.css";

const ESCAPE = "Escape";
const SUBMIT_KEY = "Enter";

/** What a form does with its state: it sends, shows why that failed, and closes when it worked. */
function useSubmit(save: () => Promise<ChangeFailure | null>, done: () => void) {
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const submit = async () => {
    setBusy(true);
    const failure = await save();
    setBusy(false);
    if (failure === null) {
      done();
    } else {
      setError(failureText(failure));
    }
  };
  return { error, busy, submit };
}

/** Escape closes the form, Ctrl+Enter sends it from a text field. */
function onFieldKey(event: KeyboardEvent, submit: () => void, cancel: () => void): void {
  if (event.key === ESCAPE) {
    event.stopPropagation();
    cancel();
  } else if (event.key === SUBMIT_KEY && event.ctrlKey) {
    event.preventDefault();
    submit();
  }
}

/** A form with its buttons and the failure of the last try. */
function Form(props: {
  onSubmit: () => void;
  onCancel: () => void;
  busy: boolean;
  error: string | null;
  label: string;
  children: JSX.Element;
}) {
  return (
    <form
      class={styles.form}
      aria-label={props.label}
      onSubmit={(event) => {
        event.preventDefault();
        props.onSubmit();
      }}
      onKeyDown={(event) => onFieldKey(event, props.onSubmit, props.onCancel)}
    >
      {props.children}
      <Show when={props.error}>
        {(message) => (
          <p class={styles.error} role="alert">
            {message()}
          </p>
        )}
      </Show>
      <div class={styles.buttons}>
        <button type="submit" class={styles.primary} disabled={props.busy}>
          {t(I18N_KEYS.ANNOTATIONS_SAVE)}
        </button>
        <button type="button" class={styles.plain} onClick={props.onCancel}>
          {t(I18N_KEYS.ANNOTATIONS_CANCEL)}
        </button>
      </div>
    </form>
  );
}

/** The name of a form: the action that opens it, or "Edit" for the edit of an existing one. */
function formLabel(editing: boolean, add: MessageKey): string {
  return t(editing ? I18N_KEYS.ANNOTATIONS_EDIT : add);
}

/** A labelled field. */
function Field(props: { label: string; children: JSX.Element }) {
  return (
    <label class={styles.field}>
      <span>{props.label}</span>
      {props.children}
    </label>
  );
}

/** The form of a note: one text field. `initial` is the text of a note that is edited. */
export function NoteForm(props: {
  initial?: string;
  save: (text: string) => Promise<ChangeFailure | null>;
  onClose: () => void;
}) {
  const [text, setText] = createSignal(props.initial ?? "");
  const state = useSubmit(() => props.save(text()), props.onClose);
  let field: HTMLTextAreaElement | undefined;
  onMount(() => field?.focus());
  return (
    <Form
      label={formLabel(props.initial !== undefined, I18N_KEYS.ANNOTATIONS_ADD_NOTE)}
      onSubmit={() => void state.submit()}
      onCancel={props.onClose}
      busy={state.busy()}
      error={state.error()}
    >
      <textarea
        ref={field}
        class={styles.text}
        rows={TEXT_ROWS}
        value={text()}
        aria-label={t(I18N_KEYS.ANNOTATIONS_NOTE_LABEL)}
        onInput={(event) => setText(event.currentTarget.value)}
      />
      <p class={styles.hint}>{t(I18N_KEYS.ANNOTATIONS_NOTE_HINT)}</p>
    </Form>
  );
}

/** The select of the status of a task. */
function StatusSelect(props: { value: TaskStatus; onChange: (status: TaskStatus) => void }) {
  return (
    <Field label={t(I18N_KEYS.ANNOTATIONS_TASK_STATUS)}>
      <select
        class={styles.input}
        value={props.value}
        onChange={(event) => props.onChange(event.currentTarget.value as TaskStatus)}
      >
        <For each={STATUSES}>
          {(status) => (
            <option value={status} selected={status === props.value}>
              {t(STATUS_LABEL[status])}
            </option>
          )}
        </For>
      </select>
    </Field>
  );
}

/** What the fields of the task form hold. */
function draftOf(
  title: string,
  text: string,
  due: string,
  assignee: string,
  status: TaskStatus,
): Draft {
  return {
    title: title.trim(),
    text: text.trim() === "" ? null : text,
    due: dueOfField(due),
    assignee: assignee.trim() === "" ? null : assignee.trim(),
    status,
  };
}

/** The fields of the task form as signals, and the draft they make. */
function useTaskFields(initial: Annotation | undefined) {
  const [title, setTitle] = createSignal(initial?.title ?? "");
  const [text, setText] = createSignal(initial?.text ?? "");
  const [due, setDue] = createSignal(dateFieldOf(initial?.due));
  const [assignee, setAssignee] = createSignal(initial?.assignee ?? "");
  const [status, setStatus] = createSignal<TaskStatus>(
    (initial?.status as TaskStatus) ?? STATUS.open,
  );
  return {
    title,
    setTitle,
    text,
    setText,
    due,
    setDue,
    assignee,
    setAssignee,
    status,
    setStatus,
    draft: () => draftOf(title(), text(), due(), assignee(), status()),
  };
}

/** The due date and the assignee of the task form, side by side. */
function DueAndAssignee(props: { fields: ReturnType<typeof useTaskFields> }) {
  const { fields } = props;
  return (
    <div class={styles.pair}>
      <Field label={t(I18N_KEYS.ANNOTATIONS_TASK_DUE)}>
        <input
          type="date"
          class={styles.input}
          value={fields.due()}
          onInput={(event) => fields.setDue(event.currentTarget.value)}
        />
      </Field>
      <Field label={t(I18N_KEYS.ANNOTATIONS_TASK_ASSIGNEE)}>
        <input
          class={styles.input}
          value={fields.assignee()}
          onInput={(event) => fields.setAssignee(event.currentTarget.value)}
        />
      </Field>
    </div>
  );
}

/** The inputs of the task form. */
function TaskFields(props: { fields: ReturnType<typeof useTaskFields>; editing: boolean }) {
  const { fields } = props;
  let first: HTMLInputElement | undefined;
  onMount(() => first?.focus());
  return (
    <>
      <Field label={t(I18N_KEYS.ANNOTATIONS_TASK_TITLE)}>
        <input
          ref={first}
          class={styles.input}
          value={fields.title()}
          onInput={(event) => fields.setTitle(event.currentTarget.value)}
        />
      </Field>
      <Field label={t(I18N_KEYS.ANNOTATIONS_TASK_TEXT)}>
        <textarea
          class={styles.text}
          rows={TEXT_ROWS}
          value={fields.text()}
          onInput={(event) => fields.setText(event.currentTarget.value)}
        />
      </Field>
      <DueAndAssignee fields={fields} />
      <Show when={props.editing}>
        <StatusSelect value={fields.status()} onChange={fields.setStatus} />
      </Show>
    </>
  );
}

/** The form of a task: title, description, due date, assignee and, when it is edited, status. */
export function TaskForm(props: {
  initial?: Annotation;
  save: (draft: Draft) => Promise<ChangeFailure | null>;
  onClose: () => void;
}) {
  const fields = useTaskFields(props.initial);
  const state = useSubmit(() => props.save(fields.draft()), props.onClose);
  return (
    <Form
      label={formLabel(props.initial !== undefined, I18N_KEYS.ANNOTATIONS_ADD_TASK)}
      onSubmit={() => void state.submit()}
      onCancel={props.onClose}
      busy={state.busy()}
      error={state.error()}
    >
      <TaskFields fields={fields} editing={props.initial !== undefined} />
    </Form>
  );
}
