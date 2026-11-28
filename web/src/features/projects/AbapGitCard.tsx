import { For, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { ABAPGIT_HEADING_ID, ABAPGIT_MISSED_HEADING_ID } from "./consts";
import { folderText, logicText } from "./abapgitText";
import type { AbapGitInfo } from "./scope";
import styles from "./AbapGitCard.module.css";

/** The card for ABAP files in a project where no abapGit layout was recognised (issue #845): it says
 * so and what to do, instead of showing nothing. */
export function AbapGitMissed() {
  return (
    <section class={styles.card} aria-labelledby={ABAPGIT_MISSED_HEADING_ID}>
      <h3 id={ABAPGIT_MISSED_HEADING_ID} class={styles.heading}>
        {t(I18N_KEYS.PROJECTS_ABAPGIT_MISSED_TITLE)}
      </h3>
      <p class={styles.note}>{t(I18N_KEYS.PROJECTS_ABAPGIT_MISSED_TEXT)}</p>
    </section>
  );
}

/** The card for an abapGit repository (ADR 0113, mock 27): where the objects lie, how many of
 * each type there are, and what ABAP does not know yet. */
export function AbapGitCard(props: { info: AbapGitInfo }) {
  return (
    <section class={styles.card} aria-labelledby={ABAPGIT_HEADING_ID}>
      <h3 id={ABAPGIT_HEADING_ID} class={styles.heading}>
        {t(I18N_KEYS.PROJECTS_ABAPGIT_TITLE)}
      </h3>
      <p class={styles.line}>
        {t(I18N_KEYS.PROJECTS_ABAPGIT_FOUND, {
          folder: folderText(props.info),
          logic: logicText(props.info),
          packages: props.info.packages,
          files: props.info.source_files,
        })}
      </p>
      <Show when={!props.info.configured}>
        <p class={styles.note}>{t(I18N_KEYS.PROJECTS_ABAPGIT_DEFAULTS)}</p>
      </Show>
      <ul class={styles.kinds} aria-label={t(I18N_KEYS.PROJECTS_ABAPGIT_KINDS)}>
        <For each={props.info.kinds}>
          {(entry) => (
            <li class={styles.kind}>
              <span class={styles.code}>{entry.kind}</span>
              <span class={styles.count}>{entry.objects}</span>
            </li>
          )}
        </For>
      </ul>
      <p class={styles.note}>{t(I18N_KEYS.PROJECTS_ABAPGIT_LIMITS)}</p>
    </section>
  );
}
