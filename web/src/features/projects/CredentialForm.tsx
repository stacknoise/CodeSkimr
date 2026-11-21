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
  /** Whether the section starts open. */
  open?: boolean;
}

/** The inputs of the form, filled in by their refs. */
interface Fields {
  name: HTMLInputElement;
  user: HTMLInputElement;
  secret: HTMLInputElement;
}

/** Empties the secret field; a secret that was typed does not stay on the page. */
function forgetSecret(fields: Fields): void {
  if (fields.secret) {
    fields.secret.value = "";
  }
}

/** Hands the login to the server; the secret leaves the page whatever the answer was. */
async function store(fields: Fields): Promise<{ id: string; problem: Failure | null }> {
  const id = fields.name.value.trim();
  try {
    return {
      id,
      problem: await storeCredential(id, fields.user.value.trim(), fields.secret.value),
    };
  } finally {
    forgetSecret(fields);
  }
}

/** Closing the section drops a secret that was typed but not stored. */
function closeHandler(forget: () => void): (event: Event) => void {
  return (event) => {
    if (!(event.currentTarget as HTMLDetailsElement).open) {
      forget();
    }
  };
}

/** Enter in a field stores the login; the form around it would add the project instead. */
function enterHandler(submit: (event: Event) => Promise<void>): (event: KeyboardEvent) => void {
  return (event) => {
    if (event.key === ENTER_KEY) {
      void submit(event);
    }
  };
}

interface FieldProps {
  setRef: (element: HTMLInputElement) => void;
  label: string;
  onEnter: (event: KeyboardEvent) => void;
  secret?: boolean;
}

/** One input of the form: a text field, or the password field for the secret. */
function Field(props: FieldProps) {
  return (
    <input
      ref={props.setRef}
      class={styles.input}
      type={props.secret ? "password" : undefined}
      aria-label={props.label}
      placeholder={props.label}
      spellcheck={false}
      autocomplete={props.secret ? "new-password" : "off"}
      onKeyDown={props.onEnter}
    />
  );
}

/** The three inputs: name, user and the secret. */
function Inputs(props: { fields: Fields; onEnter: (event: KeyboardEvent) => void }) {
  const { fields, onEnter } = props;
  return (
    <>
      <Field
        setRef={(el) => (fields.name = el)}
        label={t(I18N_KEYS.PROJECTS_CREDENTIAL_NAME)}
        onEnter={onEnter}
      />
      <Field
        setRef={(el) => (fields.user = el)}
        label={t(I18N_KEYS.PROJECTS_CREDENTIAL_USER)}
        onEnter={onEnter}
      />
      <Field
        setRef={(el) => (fields.secret = el)}
        label={t(I18N_KEYS.PROJECTS_CREDENTIAL_SECRET)}
        onEnter={onEnter}
        secret
      />
    </>
  );
}

/** What the last attempt said: the stored name or the failure. */
function Notices(props: { stored: string | null; failure: Failure | null }) {
  return (
    <>
      <Show when={props.stored}>
        {(id) => (
          <p class={styles.hint} role="status">
            {t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVED, { name: id() })}
          </p>
        )}
      </Show>
      <Show when={props.failure}>
        {(problem) => (
          <p class={cx(styles.hint, styles.error)} role="alert">
            {failureText(problem())}
          </p>
        )}
      </Show>
    </>
  );
}

/** The part of the form "New project" that stores a login for a private repository (blueprint
 * 12.2). The secret is a password field, is handed to the server once and is emptied at once. */
export function CredentialForm(props: CredentialFormProps) {
  const [busy, setBusy] = createSignal(false);
  const [failure, setFailure] = createSignal<Failure | null>(null);
  const [stored, setStored] = createSignal<string | null>(null);
  const fields = {} as Fields;
  const forget = () => forgetSecret(fields);
  // A secret that was typed but not stored does not stay behind a closed section or screen.
  onCleanup(forget);
  const submit = async (event: Event) => {
    event.preventDefault();
    if (busy()) {
      return;
    }
    setBusy(true);
    setFailure(null);
    setStored(null);
    const result = await store(fields).finally(() => setBusy(false));
    setFailure(result.problem);
    if (result.problem === null) {
      setStored(result.id);
      props.onStored(result.id);
    }
  };
  const onEnter = enterHandler(submit);
  return (
    <details class={styles.box} open={props.open} onToggle={closeHandler(forget)}>
      <summary class={styles.summary}>{t(I18N_KEYS.PROJECTS_CREDENTIAL_NEW)}</summary>
      <div class={styles.fields} role="group" aria-label={t(I18N_KEYS.PROJECTS_CREDENTIAL_NEW)}>
        <Inputs fields={fields} onEnter={onEnter} />
        <button type="button" class={styles.button} disabled={busy()} onClick={submit}>
          {busy() ? t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVING) : t(I18N_KEYS.PROJECTS_CREDENTIAL_SAVE)}
        </button>
        <p class={styles.hint}>{t(I18N_KEYS.PROJECTS_CREDENTIAL_HINT)}</p>
        <Notices stored={stored()} failure={failure()} />
      </div>
    </details>
  );
}
