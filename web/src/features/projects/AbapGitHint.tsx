import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { folderText, logicText } from "./abapgitText";
import type { AbapGitInfo } from "./scope";
import styles from "./NewProjectForm.module.css";

/** The hint of the window "New project" for a folder that is an abapGit repository (ADR 0113):
 * what the objects are counted as and where they lie. It is a hint, nothing is blocked. */
export function AbapGitHint(props: { info: AbapGitInfo }) {
  const objects = () => props.info.kinds.reduce((sum, kind) => sum + kind.objects, 0);
  return (
    <div class={styles.overlap} role="status">
      <p class={styles.overlapTitle}>{t(I18N_KEYS.PROJECTS_ABAPGIT_HINT_TITLE)}</p>
      <p>
        {t(I18N_KEYS.PROJECTS_ABAPGIT_HINT_TEXT, {
          objects: objects(),
          packages: props.info.packages,
          folder: folderText(props.info),
          logic: logicText(props.info),
        })}
      </p>
    </div>
  );
}
