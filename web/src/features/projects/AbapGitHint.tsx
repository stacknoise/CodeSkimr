import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { AbapGitInfo } from "./scope";
import styles from "./NewProjectForm.module.css";

/** The hint of the window "New project" for a folder that is an abapGit repository (ADR 0113):
 * what the objects are counted as and where they lie. It is a hint, nothing is blocked. */
export function AbapGitHint(props: { info: AbapGitInfo }) {
  const folder = () =>
    props.info.starting_folder === ""
      ? t(I18N_KEYS.PROJECTS_ABAPGIT_FOLDER_ROOT)
      : props.info.starting_folder;
  const logic = () =>
    props.info.folder_logic === "full"
      ? t(I18N_KEYS.PROJECTS_ABAPGIT_LOGIC_FULL)
      : t(I18N_KEYS.PROJECTS_ABAPGIT_LOGIC_PREFIX);
  const objects = () => props.info.kinds.reduce((sum, kind) => sum + kind.objects, 0);
  return (
    <div class={styles.overlap} role="status">
      <p class={styles.overlapTitle}>{t(I18N_KEYS.PROJECTS_ABAPGIT_HINT_TITLE)}</p>
      <p>
        {t(I18N_KEYS.PROJECTS_ABAPGIT_HINT_TEXT, {
          objects: objects(),
          packages: props.info.packages,
          folder: folder(),
          logic: logic(),
        })}
      </p>
    </div>
  );
}
