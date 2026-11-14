import { createSignal, onCleanup, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { ENTER_KEY } from "./consts";
import { storeCredential } from "./credentials";
import { failureText } from "./failure";
import type { Failure } from "./projects";
import styles from "./CredentialForm.module.css";

/** Props of the form: it tells the screen which credential was stored. */
export interface CredentialFormProps {
  onStored: (id: string) => void;
}

/** The part of the form "New project" that stores a login for a private repository (blueprint
 * 12.2). The secret is a password field, is handed to the server once and is emptied at once. */
export function CredentialForm(props: CredentialFormProps) {
  const [busy, setBusy] = createSignal(false);
  const [failure, setFailure] = createSignal<Failure | null>(null);
  const [stored, setStored] = createSignal<string | null>(null);
  let name!: HTMLInputElement;
  let user!: HTMLInputElement;
  let secret!: HTMLInputElement;
  const forget = () => {
    if (secret) {
      secret.value = "";
    }
  };
  // A secret that was typed but not stored does not stay behind a closed section or screen.
  onCleanup(forget);
  const onToggle = (event: Event) => {
    if (!(event.currentTarget as HTMLDetailsElement).open) {
      forget();
    }
  };
  /** Enter in a field stores the login; the form around it would add the project instead. */
  const onEnter = (event: KeyboardEvent) => {
    if (event.key === ENTER_KEY) {
      void submit(event);
    }
  };
  const submit = async (event: Event) => {
    event.preventDefault();
    if (busy()) {
      return;
    }
    setBusy(true);
    setFailure(null);
    setStored(null);
    const id = name.value.trim();
    let problem: Failure | null;
    try {
      problem = await storeCredential(id, user.value.trim(), secret.value);
    } finally {
      // The secret leaves the page whatever the answer was.
      forget();
      setBusy(false);
    }
    setFailure(problem);
    if (problem === null) {
      setStored(id);
      props.onStored(id);
    }
  };
  return (
    <details class={styles.box} onToggle={onToggle}>
      <summary class={styles.summary}>{t(I18N_KEYS.PROJECTS_CREDENTIAL_NEW)}</summary>
      <div class={styles.fields} role="group" aria-label={t(I18N_KEYS.PROJECTS_CREDENTIAL_NEW)}>
        <input
          ref={name}
          class={styles.input}
          aria-label={t(I18N_KEYS.PROJECTS_CREDENTIAL_NAME)}
          placeholder={t(I18N_KEYS.PROJECTS_CREDENTIAL_NAME)}
          spellcheck={false}
          autocomplete="off"
          onKeyDown={onEnter}
        />
        <input
          ref={user}
          class={styles.input}
          aria-label={t(I18N_KEYS.PROJECTS_CREDENTIAL_USER)}
          placeholder={t(I18N_KEYS.PROJECTS_CREDENTIAL_USER)}
          spellcheck={false}
          autocomplete="off"
          onKeyDown={onEnter}
        />
        <input
          ref={secret}
          class={styles.input}
          type="password"
          aria-label={t(I18N_KEYS.PROJECTS_CREDENTIAL_SECRET)}
          placeholder={t(I18N_KEYS.PROJECTS_CREDENTIAL_SECRET)}
          spellcheck={false}
          autocomplete="new-password"
          onKeyDown={onEnter}
        />
        <button type="button" class={styles.button} disabled={busy()} onClick={submit}>
          {busy() ? t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVING) : t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVE)}
        </button>
        <p class={styles.hint}>{t(I18N_KEYS.PROJECTS_CREDENTIAL_HINT)}</p>
        <Show when={stored()}>
          {(id) => (
            <p class={styles.hint} role="status">
              {t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVED, { name: id() })}
            </p>
          )}
        </Show>
        <Show when={failure()}>
          {(problem) => (
            <p class={cx(styles.hint, styles.error)} role="alert">
              {failureText(problem())}
            </p>
          )}
        </Show>
      </div>
    </details>
  );
}
