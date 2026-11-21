import { For, Show } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import type { LeftOutFile, ProjectRow } from "../projects/types";
import styles from "./LeftOut.module.css";

function reasonText(file: LeftOutFile): string {
  return t(file.kept ? I18N_KEYS.SCAN_LEFT_OUT_KEPT : I18N_KEYS.SCAN_LEFT_OUT_MISSING);
}

/** Whether no file before the one at `index` has the same code: the message of a code is said
 * once, with its first file, not under each of many. */
function firstOfItsCode(files: readonly LeftOutFile[], index: number): boolean {
  return files.findIndex((file) => file.code === files[index]?.code) === index;
}

/** The files the newest scan left out: path, code, what is shown instead and the message of the
 * code as visible text, and how many more there were than are named (ADR 0085). A snapshot from
 * before the list existed has only the count, which the card says with its own text. Nothing if
 * the scan left out none. */
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
            {(file, index) => (
              <li class={styles.item}>
                <span class={styles.path}>{file.path}</span>
                <span class={styles.code}>{file.code}</span>
                <span class={styles.reason}>{reasonText(file)}</span>
                <Show when={firstOfItsCode(files(), index())}>
                  <span class={styles.message}>{errorText(file.code as ErrorCode)}</span>
                </Show>
              </li>
            )}
          </For>
        </ul>
        <Show when={more() > 0}>
          <div class={styles.more}>
            {t(
              files().length > 0 ? I18N_KEYS.SCAN_LEFT_OUT_MORE : I18N_KEYS.SCAN_LEFT_OUT_UNNAMED,
              {
                count: more(),
              },
            )}
          </div>
        </Show>
      </div>
    </Show>
  );
}
