import { createEffect, createSignal, on } from "solid-js";
import { errorText, t } from "../shared/i18n/i18n";
import { I18N_KEYS } from "../shared/i18n/keys";
import { currentPath } from "./router";
import { ALERT_ROLE } from "../shared/ui/consts";
import { DETAILS_SEPARATOR, ERROR_CODE_FIELD, ERROR_COPIED_MS, UNEXPECTED_CODE } from "./consts";
import styles from "./UnexpectedError.module.css";

/** The details a person can hand on: the code, the screen and what the exception said. Never
 * the stack, which can hold addresses with secrets (E-05). */
export function detailsOf(error: unknown, path: string): string {
  const what = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
  return [`${ERROR_CODE_FIELD}=${UNEXPECTED_CODE}`, path, what].join(DETAILS_SEPARATOR);
}

/** Copies `text` and tells for a moment that it did. */
function copyWithNotice(text: string, setCopied: (done: boolean) => void) {
  void navigator.clipboard
    .writeText(text)
    .then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), ERROR_COPIED_MS);
    })
    .catch(() => undefined);
}

/** What the person sees instead of a screen whose drawing failed: the message, the code in small
 * monospace and the buttons "Copy details" and "Try again" (blueprint 11.5). The next screen the
 * person opens is drawn again. */
export function UnexpectedError(props: { error: unknown; reset: () => void }) {
  const [copied, setCopied] = createSignal(false);
  console.error(`${ERROR_CODE_FIELD}=${UNEXPECTED_CODE}`, props.error);
  createEffect(on(currentPath, () => props.reset(), { defer: true }));
  const copy = () => copyWithNotice(detailsOf(props.error, currentPath()), setCopied);
  return (
    <section class={styles.card} role={ALERT_ROLE}>
      <h1>{t(I18N_KEYS.APP_ERROR_TITLE)}</h1>
      <p>{errorText(UNEXPECTED_CODE)}</p>
      <code class={styles.code}>{UNEXPECTED_CODE}</code>
      <div class={styles.actions}>
        <button type="button" class={styles.button} onClick={copy}>
          {copied() ? t(I18N_KEYS.APP_ERROR_COPIED) : t(I18N_KEYS.APP_ERROR_COPY)}
        </button>
        <button type="button" class={styles.button} onClick={() => props.reset()}>
          {t(I18N_KEYS.APP_ERROR_RETRY)}
        </button>
      </div>
    </section>
  );
}
