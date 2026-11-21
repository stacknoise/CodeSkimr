import { createSignal, For, onMount, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { CredentialForm } from "../projects/CredentialForm";
import { credentialNames, refreshCredentials, removeCredential } from "../projects/credentials";
import { failureText } from "../projects/failure";
import type { Failure } from "../projects/projects";
import styles from "./SettingsScreen.module.css";

/** One stored login with the button that removes it from the keystore. */
function Row(props: { id: string; onRemove: (id: string) => void }) {
  return (
    <li class={styles.credential}>
      <span class={styles.mono}>{props.id}</span>
      <button
        type="button"
        class={styles.field}
        aria-label={t(I18N_KEYS.SETTINGS_CREDENTIALS_REMOVE_NAMED, { name: props.id })}
        onClick={() => props.onRemove(props.id)}
      >
        {t(I18N_KEYS.SETTINGS_CREDENTIALS_REMOVE)}
      </button>
    </li>
  );
}

/** The logins for private Git repositories: store a new one, see the known ones, remove one. The
 * secret never comes back; the list shows names only. */
export function CredentialsPanel() {
  const [failure, setFailure] = createSignal<Failure | null>(null);
  onMount(() => void refreshCredentials());
  const remove = async (id: string) => {
    setFailure(await removeCredential(id));
  };
  return (
    <section class={styles.panel}>
      <h2>{t(I18N_KEYS.SETTINGS_CREDENTIALS_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_CREDENTIALS_NOTE)}</p>
      <CredentialForm open onStored={() => void refreshCredentials()} />
      <Show
        when={credentialNames().length > 0}
        fallback={<p class={styles.note}>{t(I18N_KEYS.SETTINGS_CREDENTIALS_EMPTY)}</p>}
      >
        <ul class={styles.credentials}>
          <For each={credentialNames()}>{(id) => <Row id={id} onRemove={remove} />}</For>
        </ul>
      </Show>
      <Show when={failure()}>
        {(problem) => (
          <p class={styles.note} role="alert">
            {failureText(problem())}
          </p>
        )}
      </Show>
    </section>
  );
}
