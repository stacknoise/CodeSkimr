import { For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { SCAN_MODE } from "../projects/consts";
import { ago } from "../projects/format";
import type { ProjectRow } from "../projects/types";
import { RECENT_LIMIT } from "./consts";
import styles from "./Recent.module.css";

/** The projects that have a scan, the newest scan first. */
export function recentScans(rows: readonly ProjectRow[]): ProjectRow[] {
  return rows
    .filter((row) => row.scan !== null)
    .sort((a, b) => (b.scan?.created ?? 0) - (a.scan?.created ?? 0))
    .slice(0, RECENT_LIMIT);
}

function modeText(row: ProjectRow): string {
  return t(
    row.scan?.mode === SCAN_MODE.incremental
      ? I18N_KEYS.PROJECTS_SCAN_INCREMENTAL
      : I18N_KEYS.PROJECTS_SCAN_FULL,
  );
}

/** The newest scan of each project and the note on unchanged files (mock 5, right column). */
export function Recent(props: { rows: readonly ProjectRow[] }) {
  return (
    <div class={styles.card}>
      <div class={styles.title}>{t(I18N_KEYS.SCAN_RECENT_TITLE)}</div>
      <For each={recentScans(props.rows)}>
        {(row) => (
          <div class={styles.line}>
            <span>{t(I18N_KEYS.SCAN_RECENT_LINE, { name: row.name, mode: modeText(row) })}</span>
            <span class={styles.result}>
              {t(I18N_KEYS.SCAN_RECENT_RESULT, {
                files: row.scan?.files ?? 0,
                when: ago(row.scan?.created ?? 0, Date.now(), currentLanguage()),
              })}
            </span>
          </div>
        )}
      </For>
      <div class={styles.note}>{t(I18N_KEYS.SCAN_HINT_UNCHANGED)}</div>
    </div>
  );
}
