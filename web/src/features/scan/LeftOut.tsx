import { For, Show } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import type { LeftOutFile, ProjectRow } from "../projects/types";
import styles from "./LeftOut.module.css";

function reasonText(file: LeftOutFile): string {
  return t(file.kept ? I18N_KEYS.SCAN_LEFT_OUT_KEPT : I18N_KEYS.SCAN_LEFT_OUT_MISSING);
}

/** The files the newest scan left out: path, code with its message and what is shown instead,
 * and how many more there were than are named (ADR 0085). Nothing if the scan left out none. */
export function LeftOut(props: { row: ProjectRow | undefined }) {
  const files = () => props.row?.left_out ?? [];
  const more = () => props.row?.left_out_more ?? 0;
  return (
    <Show when={files().length > 0 || more() > 0}>
      <div class={styles.card}>
        <div class={styles.title}>{t(I18N_KEYS.SCAN_LEFT_OUT_TITLE)}</div>
        <div class={styles.note}>{t(I18N_KEYS.SCAN_LEFT_OUT_HINT)}</div>
        <ul class={styles.list}>
          <For each={files()}>
            {(file) => (
              <li class={styles.item}>
                <span class={styles.path}>{file.path}</span>
                <span class={styles.code} title={errorText(file.code as ErrorCode)}>
                  {file.code}
                </span>
                <span class={styles.reason}>{reasonText(file)}</span>
              </li>
            )}
          </For>
        </ul>
        <Show when={more() > 0}>
          <div class={styles.more}>{t(I18N_KEYS.SCAN_LEFT_OUT_MORE, { count: more() })}</div>
        </Show>
      </div>
    </Show>
  );
}
