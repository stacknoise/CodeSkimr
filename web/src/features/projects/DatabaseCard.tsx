import { Show } from "solid-js";
import { currentLanguage, t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { COPY, TONE } from "../../shared/ui/consts";
import { createCopier } from "../../shared/ui/copier";
import { Pill, type PillTone } from "../../shared/ui/Pill";
import { DB_STATE } from "./consts";
import { size } from "./format";
import type { ProjectRow } from "./types";
import styles from "./Overview.module.css";

type Database = NonNullable<ProjectRow["database"]>;

const LOOK: Record<Database["state"], { text: MessageKey; tone: PillTone }> = {
  [DB_STATE.ok]: { text: I18N_KEYS.PROJECTS_DB_STATE_OK, tone: TONE.good },
  [DB_STATE.missing]: { text: I18N_KEYS.PROJECTS_DB_STATE_MISSING, tone: TONE.bad },
  [DB_STATE.removing]: { text: I18N_KEYS.PROJECTS_DB_STATE_REMOVING, tone: TONE.warn },
};

/** Where the database file of the project lies and how big it is (ADR 0096, decision 3). While the
 * server does not send the field, the card says so. */
export function DatabaseCard(props: { row: ProjectRow }) {
  const copier = createCopier();
  return (
    <section class={styles.card} aria-label={t(I18N_KEYS.PROJECTS_CARD_DATABASE)}>
      <div class={styles.cardHead}>
        <h3 class={styles.cardTitle}>{t(I18N_KEYS.PROJECTS_CARD_DATABASE)}</h3>
        <Show when={props.row.database}>
          {(database) => (
            <Pill tone={LOOK[database().state].tone}>{t(LOOK[database().state].text)}</Pill>
          )}
        </Show>
      </div>
      <Show
        when={props.row.database}
        fallback={<p class={styles.muted}>{t(I18N_KEYS.PROJECTS_DB_UNAVAILABLE)}</p>}
      >
        {(database) => (
          <>
            <p class={styles.path}>{database().path}</p>
            <p class={styles.muted}>
              {t(I18N_KEYS.PROJECTS_DB_SIZE, {
                size: size(database().size_bytes, currentLanguage()),
              })}
            </p>
            <button type="button" class={styles.link} onClick={() => copier.copy(database().path)}>
              {copier.state() === COPY.copied
                ? t(I18N_KEYS.PROJECTS_DB_COPIED)
                : t(I18N_KEYS.PROJECTS_DB_COPY)}
            </button>
          </>
        )}
      </Show>
    </section>
  );
}
