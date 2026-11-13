import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import styles from "./ProjectsScreen.module.css";

/** Screen 1 is built by its own work package; until then it says how a project gets here. */
export function ProjectsScreen() {
  return (
    <section class={styles.empty}>
      <h1>{t(I18N_KEYS.PROJECTS_EMPTY_TITLE)}</h1>
      <p>{t(I18N_KEYS.PROJECTS_EMPTY_TEXT)}</p>
    </section>
  );
}
