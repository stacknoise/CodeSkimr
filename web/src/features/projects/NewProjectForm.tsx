import { createSignal, For, onMount, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { DEFAULT_BRANCH } from "./consts";
import { failureText } from "./failure";
import { CredentialForm } from "./CredentialForm";
import { credentialNames, refreshCredentials } from "./credentials";
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

/** The source field with its label (mock 1). */
function SourceField(props: NewProjectFormProps) {
  return (
    <>
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
    </>
  );
}

/** The branch field; the form reads the value when it is sent. */
function BranchField(props: { onBranch: (value: string) => void }) {
  return (
    <>
      <label for="branch" class={styles.branchLabel}>
        {t(I18N_KEYS.PROJECTS_FIELD_BRANCH)}
      </label>
      <input
        id="branch"
        class={styles.branch}
        value={DEFAULT_BRANCH}
        onInput={(event) => props.onBranch(event.currentTarget.value)}
        spellcheck={false}
        autocomplete="off"
      />
    </>
  );
}

/** The choice of a stored credential (or none). */
function CredentialSelect(props: { value: string; onChange: (id: string) => void }) {
  return (
    <>
      <label for="credential" class={styles.branchLabel}>
        {t(I18N_KEYS.PROJECTS_CREDENTIAL_LABEL)}
      </label>
      <select
        id="credential"
        class={styles.credential}
        value={props.value}
        onChange={(event) => props.onChange(event.currentTarget.value)}
      >
        <option value="">{t(I18N_KEYS.PROJECTS_CREDENTIAL_NONE)}</option>
        <For each={credentialNames()}>{(id) => <option value={id}>{id}</option>}</For>
      </select>
    </>
  );
}

/** The notice of the screen and the failure of the last call. */
function Messages(props: { notice: string | null }) {
  return (
    <>
      <Show when={props.notice}>{(text) => <p class={styles.message}>{text()}</p>}</Show>
      <Show when={projectsFailure()}>
        {(failure) => (
          <p class={cx(styles.message, styles.error)} role="alert">
            {failureText(failure())}
          </p>
        )}
      </Show>
    </>
  );
}

/** The inline form "New project" (mock 1): source, branch and the button. */
export function NewProjectForm(props: NewProjectFormProps) {
  let branch = DEFAULT_BRANCH;
  const [credential, setCredential] = createSignal("");
  onMount(() => void refreshCredentials());
  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (props.source.trim() === "" || projectAdding()) {
      return;
    }
    if (await addProject(props.source.trim(), branch.trim(), credential())) {
      props.onSource("");
    }
  };
  const stored = (id: string) => {
    void refreshCredentials();
    setCredential(id);
  };
  return (
    <form class={cx(styles.form, props.dragging && styles.dragging)} onSubmit={submit}>
      <SourceField {...props} />
      <BranchField onBranch={(value) => (branch = value)} />
      <CredentialSelect value={credential()} onChange={setCredential} />
      <button type="submit" class={styles.button} disabled={projectAdding()}>
        {projectAdding() ? t(I18N_KEYS.PROJECTS_ADDING) : t(I18N_KEYS.PROJECTS_ADD_AND_SCAN)}
      </button>
      <CredentialForm onStored={stored} />
      <Messages notice={props.notice} />
    </form>
  );
}
