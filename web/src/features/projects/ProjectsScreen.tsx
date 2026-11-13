import { createMemo, createSignal, onCleanup, onMount, Show } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { activeProject, openProject } from "../../shared/project/active";
import { count } from "./format";
import { DROP, NOT_FOUND } from "./consts";
import { readDrop } from "./drop";
import { useProjectKeys } from "./keys";
import { NewProjectForm } from "./NewProjectForm";
import { projects, projectsLoaded, refreshProjects, rescanProject } from "./projects";
import { ProjectTable } from "./ProjectTable";
import styles from "./ProjectsScreen.module.css";

/** Screen 1, the project list (blueprint B.1, mock `Main.dc.html`). */
export function ProjectsScreen() {
  const [source, setSource] = createSignal("");
  const [dragging, setDragging] = createSignal(false);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [selected, setSelected] = createSignal<number | null>(activeProject());
  let input: HTMLInputElement | undefined;

  onMount(() => void refreshProjects());
  const onDragOver = (event: DragEvent) => {
    event.preventDefault();
    setDragging(true);
  };
  const onDrop = (event: DragEvent) => {
    event.preventDefault();
    setDragging(false);
    const dropped = event.dataTransfer ? readDrop(event.dataTransfer) : { kind: DROP.empty };
    setNotice(dropped.kind === DROP.opaque ? t(I18N_KEYS.PROJECTS_DROP_WITHOUT_PATH) : null);
    if (dropped.kind === DROP.text) {
      setSource(dropped.text);
    }
    input?.focus();
  };
  const onDragEnd = () => setDragging(false);
  onMount(() => {
    window.addEventListener("dragover", onDragOver);
    window.addEventListener("drop", onDrop);
    window.addEventListener("dragleave", onDragEnd);
  });
  onCleanup(() => {
    window.removeEventListener("dragover", onDragOver);
    window.removeEventListener("drop", onDrop);
    window.removeEventListener("dragleave", onDragEnd);
  });

  const ids = createMemo(() => projects().map((row) => row.id));
  const move = (step: number) => {
    const list = ids();
    const at = list.indexOf(selected() ?? NOT_FOUND);
    const next = list[Math.min(list.length - 1, Math.max(0, at + step))];
    if (next !== undefined) {
      setSelected(next);
    }
  };
  const act = (run: (id: number) => void) => () => {
    const id = selected();
    if (id !== null) {
      run(id);
    }
  };
  useProjectKeys({
    newProject: () => input?.focus(),
    rescan: act((id) => void rescanProject(id)),
    open: act(openProject),
    move,
  });

  const types = () => projects().reduce((sum, row) => sum + row.types, 0);
  return (
    <section class={styles.screen}>
      <div>
        <h1>{t(I18N_KEYS.PROJECTS_TITLE)}</h1>
        <p class={styles.hint}>
          <Show when={projectsLoaded()}>
            {t(I18N_KEYS.PROJECTS_SUMMARY, {
              projects: projects().length,
              types: count(types(), currentLanguage()),
            })}
            {" · "}
          </Show>
          {t(I18N_KEYS.PROJECTS_HINT_KEYS)}
        </p>
      </div>
      <NewProjectForm
        source={source()}
        onSource={(value) => {
          setNotice(null);
          setSource(value);
        }}
        setInput={(element) => (input = element)}
        dragging={dragging()}
        notice={notice()}
      />
      <Show
        when={projects().length > 0}
        fallback={
          <Show when={projectsLoaded()}>
            <div class={styles.empty}>
              <h2>{t(I18N_KEYS.PROJECTS_EMPTY_TITLE)}</h2>
              <p>{t(I18N_KEYS.PROJECTS_EMPTY_TEXT)}</p>
            </div>
          </Show>
        }
      >
        <ProjectTable
          rows={projects()}
          selected={selected()}
          onSelect={setSelected}
          onOpen={(id) => {
            setSelected(id);
            openProject(id);
          }}
          onRescan={(id) => void rescanProject(id)}
        />
      </Show>
    </section>
  );
}
