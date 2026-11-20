import { createEffect, createSignal, For, onMount, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { SETTINGS_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { DEFAULT_BRANCH, FOLDER_PATH, GIT_ADDRESS, SOURCE_KIND, type SourceKind } from "./consts";
import { BrowseButton, BrowseWaiting, useBrowse } from "./BrowseButton";
import { failureText } from "./failure";
import { readFolderDialog } from "./folderDialog";
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

/** The choice between a folder and a Git repository; the other fields follow it. */
function KindSelect(props: { kind: SourceKind; onKind: (kind: SourceKind) => void }) {
  return (
    <select
      class={styles.credential}
      aria-label={t(I18N_KEYS.PROJECTS_FIELD_SOURCE_KIND)}
      value={props.kind}
      onChange={(event) => props.onKind(event.currentTarget.value as SourceKind)}
    >
      <option value={SOURCE_KIND.folder}>{t(I18N_KEYS.PROJECTS_SOURCE_FOLDER)}</option>
      <option value={SOURCE_KIND.git}>{t(I18N_KEYS.PROJECTS_SOURCE_GIT)}</option>
    </select>
  );
}

/** The source field with its label (mock 1). */
function SourceField(props: NewProjectFormProps & { kind: SourceKind }) {
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
        placeholder={t(
          props.kind === SOURCE_KIND.git
            ? I18N_KEYS.PROJECTS_SOURCE_PLACEHOLDER_GIT
            : I18N_KEYS.PROJECTS_SOURCE_PLACEHOLDER_FOLDER,
        )}
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

/** The kind of source: a folder at first; a pasted or dropped text that is clearly an address or a
 * path chooses it, the select changes it. */
function useKind(source: () => string) {
  const [kind, setKind] = createSignal<SourceKind>(SOURCE_KIND.folder);
  createEffect(() => {
    if (GIT_ADDRESS.test(source())) {
      setKind(SOURCE_KIND.git);
    } else if (FOLDER_PATH.test(source())) {
      setKind(SOURCE_KIND.folder);
    }
  });
  return [kind, setKind] as const;
}

/** The fields that only a Git source has: branch, login and the way to manage logins. */
function GitFields(props: {
  onBranch: (value: string) => void;
  credential: string;
  onCredential: (id: string) => void;
}) {
  return (
    <>
      <BranchField onBranch={props.onBranch} />
      <CredentialSelect value={props.credential} onChange={props.onCredential} />
      <button type="button" class={styles.link} onClick={() => navigate(SETTINGS_ROUTE.path)}>
        {t(I18N_KEYS.PROJECTS_CREDENTIAL_SETTINGS)}
      </button>
    </>
  );
}

/** Sends the form: the source, and for a Git source the branch and the stored login. */
function useSubmit(props: NewProjectFormProps, kind: () => SourceKind) {
  let branch = DEFAULT_BRANCH;
  const [credential, setCredential] = createSignal("");
  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (props.source.trim() === "" || projectAdding()) {
      return;
    }
    const git = kind() === SOURCE_KIND.git;
    const source = props.source.trim();
    if (await addProject(source, git ? branch.trim() : DEFAULT_BRANCH, git ? credential() : "")) {
      props.onSource("");
    }
  };
  const gitFields = () => (
    <GitFields
      onBranch={(value) => (branch = value)}
      credential={credential()}
      onCredential={setCredential}
    />
  );
  return { submit, gitFields };
}

/** The inline form "New project" (mock 1): source, branch and the button. */
export function NewProjectForm(props: NewProjectFormProps) {
  const [kind, setKind] = useKind(() => props.source);
  const browse = useBrowse(props);
  const { submit, gitFields } = useSubmit(props, kind);
  onMount(() => {
    void refreshCredentials();
    void readFolderDialog();
  });
  return (
    <form class={cx(styles.form, props.dragging && styles.dragging)} onSubmit={submit}>
      <KindSelect kind={kind()} onKind={setKind} />
      <SourceField {...props} onSource={browse.typed} kind={kind()} />
      <Show when={kind() === SOURCE_KIND.folder}>
        <BrowseButton onPicked={browse.picked} />
      </Show>
      <BrowseWaiting />
      <Show when={kind() === SOURCE_KIND.git}>{gitFields()}</Show>
      <button type="submit" class={styles.button} disabled={projectAdding()}>
        {projectAdding() ? t(I18N_KEYS.PROJECTS_ADDING) : t(I18N_KEYS.PROJECTS_ADD_AND_SCAN)}
      </button>
      <Show when={browse.taken()}>
        <p class={styles.message}>{t(I18N_KEYS.PROJECTS_BROWSE_DONE)}</p>
      </Show>
      <Messages notice={props.notice} />
    </form>
  );
}
