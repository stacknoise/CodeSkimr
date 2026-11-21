import { createSignal } from "solid-js";
import type { ErrorCode } from "../api/error-codes.generated";
import { I18N_KEYS } from "../i18n/keys";
import { t } from "../i18n/i18n";
import { COPIED_MS, DETAILS_SEPARATOR } from "./consts";
import styles from "./ErrorDetails.module.css";

/** The code of an error in small monospace type, and the button "Copy details" that puts the code
 * and the message on the clipboard, so that a report can quote them (blueprint 11.5). */
export function ErrorDetails(props: { code: ErrorCode; message: string }) {
  const [copied, setCopied] = createSignal(false);
  const copy = () => {
    const details = `${props.code}${DETAILS_SEPARATOR}${props.message}`;
    void navigator.clipboard
      .writeText(details)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), COPIED_MS);
      })
      .catch(() => undefined);
  };
  return (
    <div class={styles.details}>
      <code class={styles.code}>{props.code}</code>
      <button type="button" class={styles.copy} onClick={copy}>
        {copied() ? t(I18N_KEYS.APP_ERROR_DETAILS_COPIED) : t(I18N_KEYS.APP_ERROR_COPY_DETAILS)}
      </button>
    </div>
  );
}
