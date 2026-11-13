import { Show } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEFAULT_BRANCH } from "./consts";
import { addProject, projectAdding, projectsFailure, type Failure } from "./projects";
import styles from "./NewProjectForm.module.css";

/** Props of the form: the source field belongs to the screen, which fills it on a drop and
 * focuses it on the key N. */
export interface NewProjectFormProps {
  source: string;
  onSource: (value: string) => void;
  setInput: (element: HTMLInputElement) => void;
  dragging: boolean;
  notice: string | null;
}

/** The text of a failure: the registered message, or the general one without a code. */
export function failureText(failure: Failure): string {
  return failure.code === null ? t(I18N_KEYS.PROJECTS_LOAD_FAILED) : errorText(failure.code);
}

/** The inline form "New project" (mock 1): source, branch and the button. */
export function NewProjectForm(props: NewProjectFormProps) {
  let branch = DEFAULT_BRANCH;
  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (props.source.trim() === "" || projectAdding()) {
      return;
    }
    if (await addProject(props.source.trim(), branch.trim())) {
      props.onSource("");
    }
  };
  return (
    <form class={cx(styles.form, props.dragging && styles.dragging)} onSubmit={submit}>
      <label for="source" class={styles.title}>
        {t(I18N_KEYS.PROJECTS_NEW_TITLE)}
      </label>
      <input
        id="source"
        ref={props.setInput}
        class={styles.source}
        value={props.source}
        placeholder={t(I18N_KEYS.PROJECTS_SOURCE_PLACEHOLDER)}
        onInput={(event) => props.onSource(event.currentTarget.value)}
        spellcheck={false}
        autocomplete="off"
      />
      <label for="branch" class={styles.branchLabel}>
        {t(I18N_KEYS.PROJECTS_FIELD_BRANCH)}
      </label>
      <input
        id="branch"
        class={styles.branch}
        value={DEFAULT_BRANCH}
        onInput={(event) => (branch = event.currentTarget.value)}
        spellcheck={false}
        autocomplete="off"
      />
      <button type="submit" class={styles.button} disabled={projectAdding()}>
        {projectAdding() ? t(I18N_KEYS.PROJECTS_ADDING) : t(I18N_KEYS.PROJECTS_ADD_AND_SCAN)}
      </button>
      <Show when={props.notice}>{(text) => <p class={styles.message}>{text()}</p>}</Show>
      <Show when={projectsFailure()}>
        {(failure) => (
          <p class={cx(styles.message, styles.error)} role="alert">
            {failureText(failure())}
          </p>
        )}
      </Show>
    </form>
  );
}
