import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { Overlap } from "./overlap";
import styles from "./NewProjectForm.module.css";

/** The hint of the window "New project" when the folder lies inside a project (mock 26, c1):
 * covering it there is the default, adding it as a project of its own stays possible. It is a
 * hint, no block (ADR 0102). */
export function OverlapHint(props: {
  overlap: Overlap;
  own: boolean;
  onOwn: (own: boolean) => void;
}) {
  const project = () => props.overlap.project.name;
  return (
    <fieldset class={styles.overlap}>
      <legend class={styles.overlapTitle}>
        {t(I18N_KEYS.PROJECTS_SCOPE_HINT_TITLE, { project: project() })}
      </legend>
      <p>{t(I18N_KEYS.PROJECTS_SCOPE_HINT_TEXT, { folder: `${props.overlap.folder}/` })}</p>
      <label class={styles.overlapChoice}>
        <input
          type="radio"
          name="overlap"
          checked={!props.own}
          onChange={() => props.onOwn(false)}
        />
        {t(I18N_KEYS.PROJECTS_SCOPE_HINT_INSIDE, { project: project() })}
      </label>
      <label class={styles.overlapChoice}>
        <input type="radio" name="overlap" checked={props.own} onChange={() => props.onOwn(true)} />
        {t(I18N_KEYS.PROJECTS_SCOPE_HINT_OWN)}
      </label>
    </fieldset>
  );
}
