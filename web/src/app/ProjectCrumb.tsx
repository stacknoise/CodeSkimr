import { Show } from "solid-js";
import { currentLanguage, t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { activeProject } from "../shared/project/active";
import { ICON_BRANCH, ICON_SIZE_SMALL } from "../shared/ui/consts";
import { Icon } from "../shared/ui/Icon";
import { ago } from "../features/projects/format";
import { SOURCE_KIND } from "../features/projects/consts";
import { projects } from "../features/projects/projects";
import type { ProjectRow } from "../features/projects/types";
import { CHIP_SEPARATOR, PROJECTS_ROUTE } from "./consts";
import { Crumb } from "./Frame";
import styles from "./App.module.css";

/** What the chip says: the branch of a Git source, or the working folder, and how old the
 * newest scan is. */
function chipText(project: ProjectRow): string {
  const source =
    project.kind === SOURCE_KIND.git && project.branch
      ? project.branch
      : t(I18N_KEYS.APP_CHIP_FOLDER);
  return project.scan
    ? `${source}${CHIP_SEPARATOR}${ago(project.scan.created, Date.now(), currentLanguage())}`
    : source;
}

/** The project that the screens work on, as the top bar shows it: breadcrumb and chip
 * (blueprint 15.2). Nothing is shown until a project is open. */
export function ProjectCrumb() {
  const project = () => projects().find((row) => row.id === activeProject());
  return (
    <Show when={project()}>
      {(row) => (
        <>
          <Crumb name={row().name} href={PROJECTS_ROUTE.path} />
          <span class={styles.chip}>
            <Icon name={ICON_BRANCH} size={ICON_SIZE_SMALL} />
            {chipText(row())}
          </span>
        </>
      )}
    </Show>
  );
}
