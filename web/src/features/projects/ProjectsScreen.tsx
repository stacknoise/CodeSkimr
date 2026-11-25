import { createEffect, createSignal, on, onCleanup, onMount, Show, untrack } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { CANCEL_KEY } from "../scan/consts";
import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { activeProject, closeProject, openProject } from "../../shared/project/active";
import { useProjectKeys } from "./keys";
import { level2Of, loadLevel2 } from "./level2";
import { FailureNotice, NewProjectForm } from "./NewProjectForm";
import {
  checkTools,
  projects,
  projectsLoaded,
  refreshProjects,
  cancelRename,
  cancelRenameOf,
  removeProject,
  rescanProject,
} from "./projects";
import { ProjectList } from "./ProjectList";
import { ProjectPage } from "./ProjectPage";
import { closeRemoveQuestion } from "./RemoveAction";
import { useDropSource } from "./useDropSource";
import { useSelection } from "./useSelection";
import styles from "./ProjectsScreen.module.css";

function Empty() {
  return (
    <Show when={projectsLoaded()}>
      <div class={styles.empty}>
        <h2>{t(I18N_KEYS.PROJECTS_EMPTY_TITLE)}</h2>
        <p>{t(I18N_KEYS.PROJECTS_EMPTY_TEXT)}</p>
      </div>
    </Show>
  );
}

/** Makes the project the active one and shows its structure. */
function openInExplorer(id: number): void {
  openProject(id);
  navigate(EXPLORER_ROUTE.path);
}

/** Whoever looks at a project sees what the machine has now, not what it had at the last scan. */
function useToolsCheck(selected: () => number | null): void {
  createEffect(
    on(selected, (id) => {
      if (id !== null) {
        void checkTools(id);
      }
    }),
  );
}

/** Reads the level 2 state of every project in the list, and again of the one that is selected. */
function useLevel2States(selected: () => number | null): void {
  createEffect(() => {
    const ids = projects().map((row) => row.id);
    untrack(() => {
      for (const id of ids.filter((known) => level2Of(known).status === null)) {
        void loadLevel2(id);
      }
    });
  });
  createEffect(
    on(selected, (id) => {
      if (id !== null) {
        void loadLevel2(id);
      }
    }),
  );
}

/** Removes a project: it is closed and no longer selected before its data goes. */
function removeSelected(selection: ReturnType<typeof useSelection>) {
  return async (id: number) => {
    closeProject(id);
    cancelRenameOf(id);
    if (selection.selected() === id) {
      selection.setSelected(null);
    }
    return removeProject(id);
  };
}

/** The form "New project": open from the button, the key N, a dropped folder, or with no project. */
function useForm() {
  const [source, setSource] = createSignal("");
  const [open, setOpen] = createSignal(false);
  let input: HTMLInputElement | undefined;
  const show = () => {
    setOpen(true);
    queueMicrotask(() => input?.focus());
  };
  // Closing the window empties the field, so it always opens empty.
  const close = () => {
    setOpen(false);
    setSource("");
  };
  useLayer(open);
  useKeydown((event) => {
    if (open() && event.key === CANCEL_KEY) {
      close();
    }
  });
  const drop = useDropSource(setSource, show);
  const onSource = (value: string) => {
    drop.clearNotice();
    setSource(value);
  };
  return {
    source,
    onSource,
    open,
    close,
    show,
    drop,
    setInput: (el: HTMLInputElement) => (input = el),
  };
}

/** The selection with the reads and the keys that belong to it. */
function useScreen(form: ReturnType<typeof useForm>) {
  const selection = useSelection(projects, activeProject());
  onMount(() => void refreshProjects());
  onCleanup(closeRemoveQuestion);
  // An open name edit does not wait for the next visit.
  onCleanup(cancelRename);
  useToolsCheck(selection.selected);
  useLevel2States(selection.selected);
  // A project that was chosen or whose scan started closes the form.
  createEffect(on(selection.selected, form.close, { defer: true }));
  useProjectKeys({
    newProject: form.show,
    rescan: selection.act((id) => void rescanProject(id)),
    open: selection.act(openInExplorer),
    move: selection.move,
  });
  return selection;
}

/** The small window "New project"; with no project it cannot be closed. */
function FormWindow(props: { form: ReturnType<typeof useForm>; closable: boolean }) {
  return (
    <NewProjectForm
      source={props.form.source()}
      onSource={props.form.onSource}
      setInput={props.form.setInput}
      dragging={props.form.drop.dragging()}
      notice={props.form.drop.notice()}
      onClose={props.closable ? props.form.close : undefined}
    />
  );
}

/** Screen 1, the projects (blueprint B.1, mock 23, ADR 0096): the list on the left with the small
 * window "New project" at its button (mock 24), and the page of the selected project on the right. */
export function ProjectsScreen() {
  const form = useForm();
  const selection = useScreen(form);
  const selectedRow = () => projects().find((row) => row.id === selection.selected());
  const formShown = () => form.open() || (projectsLoaded() && projects().length === 0);
  return (
    <section class={styles.screen}>
      <div class={styles.side}>
        <ProjectList
          rows={projects()}
          selected={selection.selected()}
          onSelect={selection.setSelected}
          onOpen={openInExplorer}
          onNew={form.show}
        />
        <p class={styles.hint}>{t(I18N_KEYS.PROJECTS_HINT_KEYS)}</p>
        <Show when={formShown()}>
          <FormWindow form={form} closable={projects().length > 0} />
        </Show>
      </div>
      <div class={styles.main}>
        <Show when={!formShown()}>
          <FailureNotice />
        </Show>
        <Show when={projects().length === 0}>
          <Empty />
        </Show>
        <Show when={selectedRow()}>
          {(row) => <ProjectPage row={row()} onRemove={removeSelected(selection)} />}
        </Show>
      </div>
    </section>
  );
}
