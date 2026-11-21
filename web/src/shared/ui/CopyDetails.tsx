import { createSignal, Show } from "solid-js";
import { t } from "../i18n/i18n";
import { I18N_KEYS } from "../i18n/keys";
import { COPIED_MS, DETAILS_ROWS } from "./consts";
import styles from "./CopyDetails.module.css";

/** The button "Copy details". It puts `text` on the clipboard and says so for a moment. Where the
 * browser has no clipboard (an address that is not secure) or refuses it, the text is shown in a
 * box that can be selected, so the details are never lost and a failing note does not fail again. */
export function CopyDetails(props: { text: string }) {
  const [copied, setCopied] = createSignal(false);
  const [shown, setShown] = createSignal(false);
  const copy = () => {
    if (navigator.clipboard === undefined) {
      setShown(true);
      return;
    }
    navigator.clipboard
      .writeText(props.text)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), COPIED_MS);
      })
      .catch(() => setShown(true));
  };
  return (
    <>
      <button type="button" class={styles.button} onClick={copy}>
        {copied() ? t(I18N_KEYS.APP_ERROR_COPIED) : t(I18N_KEYS.APP_ERROR_COPY)}
      </button>
      <Show when={shown()}>
        <textarea
          class={styles.details}
          readonly
          rows={DETAILS_ROWS}
          aria-label={t(I18N_KEYS.APP_ERROR_DETAILS)}
          value={props.text}
          onFocus={(event) => event.currentTarget.select()}
        />
      </Show>
    </>
  );
}
