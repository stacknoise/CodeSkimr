import { createSignal, onMount, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject, openProject } from "../../shared/project/active";
import { count } from "./format";
import { useProjectKeys } from "./keys";
import { NewProjectForm } from "./NewProjectForm";
import { projects, projectsLoaded, refreshProjects, rescanProject } from "./projects";
import { ProjectTable } from "./ProjectTable";
import { useDropSource } from "./useDropSource";
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

/** Screen 1, the project list (blueprint B.1, mock `Main.dc.html`). */
export function ProjectsScreen() {
  const [source, setSource] = createSignal("");
  let input: HTMLInputElement | undefined;
  const drop = useDropSource(setSource, () => input?.focus());
  const selection = useSelection(projects, activeProject());
  onMount(() => void refreshProjects());
  useProjectKeys({
    newProject: () => input?.focus(),
    rescan: selection.act((id) => void rescanProject(id)),
    open: selection.act(openProject),
    move: selection.move,
  });
  return (
    <section class={styles.screen}>
      <Header />
      <NewProjectForm
        source={source()}
        onSource={(value) => {
          drop.clearNotice();
          setSource(value);
        }}
        setInput={(element) => (input = element)}
        dragging={drop.dragging()}
        notice={drop.notice()}
      />
      <Show when={projects().length > 0} fallback={<Empty />}>
        <ProjectTable
          rows={projects()}
          selected={selection.selected()}
          onSelect={selection.setSelected}
          onOpen={(id) => {
            selection.setSelected(id);
            openProject(id);
          }}
          onRescan={(id) => void rescanProject(id)}
        />
      </Show>
    </section>
  );
}
