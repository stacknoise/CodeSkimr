import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from "solid-js";
import { Portal } from "solid-js/web";
import { t } from "../../shared/i18n/i18n";
import { useDialogFocus } from "../../shared/ui/dialogFocus";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { SETTINGS_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import {
  DEFAULT_BRANCH,
  FOLDER_PATH,
  GIT_ADDRESS,
  NETWORK_PATH,
  NEW_PROJECT_TITLE_ID,
  PROJECT_TAB,
  SOURCE_KIND,
  SUGGEST_DELAY_MS,
  type SourceKind,
} from "./consts";
import { openProject } from "../../shared/project/active";
import { AbapGitHint } from "./AbapGitHint";
import { useInspected } from "./inspect";
import { OverlapHint } from "./OverlapHint";
import { overlapOf, type Overlap } from "./overlap";
import { selectProject } from "./page";
import { BrowseButton, BrowseWaiting, useBrowse } from "./BrowseButton";
import { failureText } from "./failure";
import { readFolderDialog } from "./folderDialog";
import { credentialNames, refreshCredentials } from "./credentials";
import {
  addProject,
  projectAdding,
  projects,
  projectsFailure,
  suggestNameRetrying,
  type Failure,
} from "./projects";
import styles from "./NewProjectForm.module.css";

/** Props of the form: the source field belongs to the screen, which fills it on a drop and
 * focuses it on the key N. */
export interface NewProjectFormProps {
  source: string;
  onSource: (value: string) => void;
  setInput: (element: HTMLInputElement) => void;
  dragging: boolean;
  notice: string | null;
  /** Closes the window; absent while the list is empty, when the window stays. */
  onClose?: (() => void) | undefined;
}

/** The choice between a folder and a Git repository (mock 24); the other fields follow it. */
function KindSwitch(props: { kind: SourceKind; onKind: (kind: SourceKind) => void }) {
  const choices = [
    [SOURCE_KIND.folder, I18N_KEYS.PROJECTS_SOURCE_FOLDER],
    [SOURCE_KIND.git, I18N_KEYS.PROJECTS_SOURCE_GIT],
  ] as const;
  return (
    <div
      class={styles.segment}
      role="radiogroup"
      aria-label={t(I18N_KEYS.PROJECTS_FIELD_SOURCE_KIND)}
    >
      <For each={choices}>
        {([kind, text]) => (
          <button
            type="button"
            role="radio"
            aria-checked={props.kind === kind}
            class={cx(styles.choice, props.kind === kind && styles.chosen)}
            onClick={() => props.onKind(kind)}
          >
            {t(text)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The source field with its label: "Folder" or "Repository address". */
function SourceField(props: NewProjectFormProps & { kind: SourceKind; children: JSX.Element }) {
  const git = () => props.kind === SOURCE_KIND.git;
  return (
    <div class={styles.field}>
      <label for="source" class={styles.label}>
        {t(git() ? I18N_KEYS.PROJECTS_FIELD_ADDRESS : I18N_KEYS.PROJECTS_SOURCE_FOLDER)}
      </label>
      <div class={styles.pair}>
        <input
          id="source"
          ref={props.setInput}
          class={styles.source}
          value={props.source}
          placeholder={t(
            git()
              ? I18N_KEYS.PROJECTS_SOURCE_PLACEHOLDER_GIT
              : I18N_KEYS.PROJECTS_SOURCE_PLACEHOLDER_FOLDER,
          )}
          onInput={(event) => props.onSource(event.currentTarget.value)}
          spellcheck={false}
          autocomplete="off"
        />
        {props.children}
      </div>
    </div>
  );
}

/** The branch field; the form reads the value when it is sent. */
function BranchField(props: { onBranch: (value: string) => void }) {
  return (
    <div class={styles.field}>
      <label for="branch" class={styles.label}>
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
    </div>
  );
}

/** The choice of a stored credential (or none). */
function CredentialSelect(props: { value: string; onChange: (id: string) => void }) {
  return (
    <div class={styles.field}>
      <label for="credential" class={styles.label}>
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
    </div>
  );
}

/** The failure of the last call, in words; the screen shows it also while the form is closed. */
export function FailureNotice() {
  return (
    <Show when={projectsFailure()}>
      {(failure) => (
        <p class={cx(styles.message, styles.error)} role="alert">
          {failureText(failure())}
        </p>
      )}
    </Show>
  );
}

/** The notice of the screen and the failure of the last call. */
function Messages(props: { notice: string | null }) {
  return (
    <>
      <Show when={props.notice}>{(text) => <p class={styles.message}>{text()}</p>}</Show>
      <FailureNotice />
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
      <div class={styles.pair}>
        <BranchField onBranch={props.onBranch} />
        <CredentialSelect value={props.credential} onChange={props.onCredential} />
      </div>
      <button type="button" class={styles.link} onClick={() => navigate(SETTINGS_ROUTE.path)}>
        {t(I18N_KEYS.PROJECTS_CREDENTIAL_SETTINGS)}
      </button>
    </>
  );
}

/** The display name: filled with the name that the source would get until the user types one.
 * Only a typed name is sent, so adding a known source again never resets its name. */
function useName(source: () => string) {
  const [name, setName] = createSignal("");
  const [typed, setTyped] = createSignal(false);
  let asked = 0;
  createEffect(() => {
    const text = source().trim();
    const ticket = ++asked;
    if (text === "") {
      setTyped(false);
    }
    // A network path (`\\server\share`) would make the server reach out for every pause in
    // typing; it gets its name when it is added.
    const wanted = text !== "" && !NETWORK_PATH.test(text);
    const timer = setTimeout(() => {
      void (
        wanted ? suggestNameRetrying(text, () => ticket === asked) : Promise.resolve(null)
      ).then((suggested) => {
        if (ticket === asked && !typed()) {
          setName(suggested ?? "");
        }
      });
    }, SUGGEST_DELAY_MS);
    onCleanup(() => clearTimeout(timer));
  });
  const type = (value: string) => {
    setTyped(value !== "");
    setName(value);
  };
  const sent = () => (typed() ? name().trim() : "");
  return { name, type, sent };
}

/** The optional display name next to the source. */
function NameField(props: { value: string; onType: (value: string) => void }) {
  return (
    <div class={styles.field}>
      <label for="project-name" class={styles.label}>
        {t(I18N_KEYS.PROJECTS_FIELD_NAME)}{" "}
        <span class={styles.optional}>({t(I18N_KEYS.PROJECTS_FIELD_OPTIONAL)})</span>
      </label>
      <input
        id="project-name"
        class={styles.name}
        value={props.value}
        title={t(I18N_KEYS.PROJECTS_NAME_HINT)}
        onInput={(event) => props.onType(event.currentTarget.value)}
        spellcheck={false}
        autocomplete="off"
      />
    </div>
  );
}

/** Whether the window that calls this is still there. The window can close while the add runs on
 * (the first scan report of the new project selects it, which closes the window). A window opened
 * since then is another one and keeps what the user types; only a window that is still the one
 * that sent the form closes after it. */
function useAlive(): () => boolean {
  let alive = true;
  onCleanup(() => (alive = false));
  return () => alive;
}

/** The folder lies in a project that covers it: nothing is added, the tab Scope of that project
 * opens. */
function showCovering(props: NewProjectFormProps, inside: Overlap): void {
  props.onSource("");
  props.onClose?.();
  openProject(inside.project.id);
  selectProject(inside.project.id, PROJECT_TAB.scope);
}

/** Sends the form: the source, and for a Git source the branch and the stored login. */
function useSubmit(
  props: NewProjectFormProps,
  kind: () => SourceKind,
  name: () => string,
  covered: () => Overlap | null,
) {
  let branch = DEFAULT_BRANCH;
  const alive = useAlive();
  const [credential, setCredential] = createSignal("");
  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (props.source.trim() === "" || projectAdding()) {
      return;
    }
    const inside = covered();
    if (inside !== null) {
      showCovering(props, inside);
      return;
    }
    const git = kind() === SOURCE_KIND.git;
    const [wanted, login] = git ? [branch.trim(), credential()] : [DEFAULT_BRANCH, ""];
    const added = await addProject(props.source.trim(), wanted, login, name());
    if (added && alive()) {
      // The window closes (and empties the field) once the project is added.
      props.onSource("");
      props.onClose?.();
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

/** The foot of the window: the keys, "Cancel" and the button that adds. */
function Foot(props: { onClose: (() => void) | undefined; covered: boolean }) {
  return (
    <div class={styles.foot}>
      <span class={styles.keys}>{t(I18N_KEYS.PROJECTS_NEW_KEYS)}</span>
      <Show when={props.onClose}>
        <button type="button" class={styles.cancel} onClick={() => props.onClose?.()}>
          {t(I18N_KEYS.PROJECTS_REMOVE_CANCEL)}
        </button>
      </Show>
      <button type="submit" class={styles.button} disabled={projectAdding()}>
        {projectAdding()
          ? t(I18N_KEYS.PROJECTS_ADDING)
          : t(props.covered ? I18N_KEYS.PROJECTS_SCOPE_HINT_NEXT : I18N_KEYS.PROJECTS_ADD_AND_SCAN)}
      </button>
    </div>
  );
}

/** The fields of the window: source with "Browse", name, and the Git fields of a Git source. */
function Body(props: {
  form: NewProjectFormProps;
  kind: SourceKind;
  name: ReturnType<typeof useName>;
  gitFields: () => JSX.Element;
  overlap: ReturnType<typeof useOverlap>;
}) {
  const browse = useBrowse(props.form);
  const inspected = useInspected(
    () => props.form.source,
    () => props.kind === SOURCE_KIND.folder,
  );
  return (
    <div class={styles.body}>
      <SourceField {...props.form} onSource={browse.typed} kind={props.kind}>
        <Show when={props.kind === SOURCE_KIND.folder}>
          <BrowseButton onPicked={browse.picked} />
        </Show>
      </SourceField>
      <BrowseWaiting />
      <Show when={browse.taken()}>
        <p class={styles.message}>{t(I18N_KEYS.PROJECTS_BROWSE_DONE)}</p>
      </Show>
      <NameField value={props.name.name()} onType={props.name.type} />
      <Show when={props.kind === SOURCE_KIND.git}>{props.gitFields()}</Show>
      <Show when={props.overlap.overlap()}>
        {(found) => (
          <OverlapHint overlap={found()} own={props.overlap.own()} onOwn={props.overlap.setOwn} />
        )}
      </Show>
      <Show when={inspected()}>{(info) => <AbapGitHint info={info()} />}</Show>
      <Messages notice={props.form.notice} />
    </div>
  );
}

/** Whether the typed folder lies inside a project, and whether the user chose to add it anyway. */
function useOverlap(kind: () => SourceKind, source: () => string) {
  const [own, setOwn] = createSignal(false);
  const overlap = createMemo(() =>
    kind() === SOURCE_KIND.folder ? overlapOf(source(), projects()) : null,
  );
  const covered = () => (own() ? null : overlap());
  return { overlap, own, setOwn, covered };
}

/** The modal window "New project" (mock 24, variant C as a centred dialog): source, name, and for
 * a Git source branch and login. Esc and "Cancel" close it; with no project it stays open. */
export function NewProjectForm(props: NewProjectFormProps) {
  const [kind, setKind] = useKind(() => props.source);
  const name = useName(() => props.source);
  const found = useOverlap(kind, () => props.source);
  const { submit, gitFields } = useSubmit(props, kind, name.sent, found.covered);
  onMount(() => {
    void refreshCredentials();
    void readFolderDialog();
  });
  let dialog: HTMLFormElement | undefined;
  useDialogFocus(() => dialog);
  return (
    <Portal>
      <div class={styles.scrim} />
      <form
        ref={dialog}
        class={cx(styles.form, props.dragging && styles.dragging)}
        role="dialog"
        aria-modal="true"
        aria-labelledby={NEW_PROJECT_TITLE_ID}
        onSubmit={submit}
      >
        <div class={styles.head}>
          <h2 id={NEW_PROJECT_TITLE_ID} class={styles.title}>
            {t(I18N_KEYS.PROJECTS_NEW_TITLE)}
          </h2>
          <KindSwitch kind={kind()} onKind={setKind} />
        </div>
        <Body form={props} kind={kind()} name={name} gitFields={gitFields} overlap={found} />
        <Foot onClose={props.onClose} covered={found.covered() !== null} />
      </form>
    </Portal>
  );
}
