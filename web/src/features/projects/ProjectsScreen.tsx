import { createEffect, createSignal, on, onCleanup, onMount, Show, untrack } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { EXPLORER_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";
import { activeProject, openProject } from "../../shared/project/active";
import { count } from "./format";
import { useProjectKeys } from "./keys";
import { Level2Panel } from "./Level2Panel";
import { level2Of, loadLevel2 } from "./level2";
import { NewProjectForm } from "./NewProjectForm";
import {
  checkTools,
  projects,
  projectsLoaded,
  refreshProjects,
  removeProject,
  rescanProject,
} from "./projects";
import { ProjectTable } from "./ProjectTable";
import { closeRemoveQuestion } from "./RemoveAction";
import { ScanPanel } from "./ScanPanel";
import { ToolsPanel } from "./ToolsPanel";
import { useDropSource } from "./useDropSource";
import type { ProjectRow } from "./types";
import { useSelection } from "./useSelection";
import styles from "./ProjectsScreen.module.css";

function Header() {
  const types = () => projects().reduce((sum, row) => sum + row.types, 0);
  return (
    <div>
      <h1>{t(I18N_KEYS.PROJECTS_TITLE)}</h1>
      <p class={styles.hint}>
        {projectsLoaded()
          ? t(I18N_KEYS.PROJECTS_HEADER, {
              summary: t(I18N_KEYS.PROJECTS_SUMMARY, {
                projects: projects().length,
                types: count(types(), currentLanguage()),
              }),
              hints: t(I18N_KEYS.PROJECTS_HINT_KEYS),
            })
          : t(I18N_KEYS.PROJECTS_HINT_KEYS)}
      </p>
    </div>
  );
}

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

/** The project table wired to the selection of the screen. */
function Table(props: { selection: ReturnType<typeof useSelection> }) {
  const { selection } = props;
  return (
    <ProjectTable
      rows={projects()}
      selected={selection.selected()}
      onSelect={selection.setSelected}
      onOpen={(id) => {
        selection.setSelected(id);
        openInExplorer(id);
      }}
      onRescan={(id) => void rescanProject(id)}
      onRemove={async (id) => {
        const removed = await removeProject(id);
        if (removed && selection.selected() === id) {
          selection.setSelected(null);
        }
        return removed;
      }}
    />
  );
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

/** The sections of the selected project: level 2 and the tool check. */
function Panels(props: { row: ProjectRow | undefined }) {
  return (
    <Show when={props.row}>
      {(row) => (
        <>
          <Level2Panel row={row()} />
          <ScanPanel row={row()} />
          <ToolsPanel row={row()} />
        </>
      )}
    </Show>
  );
}

/** Screen 1, the project list (blueprint B.1, mock `Main.dc.html`). */
export function ProjectsScreen() {
  const [source, setSource] = createSignal("");
  let input: HTMLInputElement | undefined;
  const drop = useDropSource(setSource, () => input?.focus());
  const selection = useSelection(projects, activeProject());
  onMount(() => void refreshProjects());
  onCleanup(closeRemoveQuestion);
  useToolsCheck(selection.selected);
  useLevel2States(selection.selected);
  const selectedRow = () => projects().find((row) => row.id === selection.selected());
  useProjectKeys({
    newProject: () => input?.focus(),
    rescan: selection.act((id) => void rescanProject(id)),
    open: selection.act(openInExplorer),
    move: selection.move,
  });
  const onSource = (value: string) => {
    drop.clearNotice();
    setSource(value);
  };
  return (
    <section class={styles.screen}>
      <Header />
      <NewProjectForm
        source={source()}
        onSource={onSource}
        setInput={(element) => (input = element)}
        dragging={drop.dragging()}
        notice={drop.notice()}
      />
      <Show when={projects().length > 0} fallback={<Empty />}>
        <Table selection={selection} />
      </Show>
      <Panels row={selectedRow()} />
    </section>
  );
}
