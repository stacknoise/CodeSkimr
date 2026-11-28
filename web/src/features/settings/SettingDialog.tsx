import { Show, createSignal } from "solid-js";
import { Portal } from "solid-js/web";
import { ApiError } from "../../shared/api/client";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { TONE } from "../../shared/ui/consts";
import { useDialogFocus } from "../../shared/ui/dialogFocus";
import { Pill } from "../../shared/ui/Pill";
import {
  EDIT,
  HINT_SEPARATOR,
  ORIGIN,
  SETTING_CANCEL_KEY,
  SETTING_DIALOG_FIELD_ID,
  SETTING_DIALOG_TITLE_ID,
  SETTING_SUBMIT_KEY,
  SWITCH_OFF,
  SWITCH_ON,
  VALUE_KIND,
  VALUE_REFUSED_CODE,
} from "./consts";
import { displayValue, saveSettingValue, type EffectiveSetting } from "./effective";
import { KIND_TEXTS, kindOf, parseEntry } from "./settingEdit";
import styles from "./SettingDialog.module.css";

/** What the dialog says after a failed save: a registered code if the server named one. */
function failureText(error: unknown, kindText: string): string {
  if (error instanceof ApiError && error.code !== null) {
    return error.code === VALUE_REFUSED_CODE
      ? t(I18N_KEYS.SETTINGS_SETTING_INVALID, { kind: kindText })
      : errorText(error.code);
  }
  return t(I18N_KEYS.SETTINGS_SETTING_FAILED);
}

/** From when the new value counts, with the reason. */
function Applies(props: { start: boolean }) {
  return (
    <div class={styles.applies}>
      <Pill tone={TONE.accent}>
        {props.start
          ? t(I18N_KEYS.SETTINGS_SETTING_APPLIES_START)
          : t(I18N_KEYS.SETTINGS_SETTING_APPLIES_SCAN)}
      </Pill>
      <span class={styles.hint}>
        {props.start
          ? t(I18N_KEYS.SETTINGS_SETTING_APPLIES_START_NOTE)
          : t(I18N_KEYS.SETTINGS_SETTING_APPLIES_SCAN_NOTE)}
      </span>
    </div>
  );
}

/** The field of the new value: a choice for a switch, a text field for a number. */
function Field(props: {
  switchKind: boolean;
  text: string;
  onText: (text: string) => void;
  onSubmit: () => void;
  ref: (element: HTMLInputElement | HTMLSelectElement) => void;
}) {
  return props.switchKind ? (
    <select
      id={SETTING_DIALOG_FIELD_ID}
      ref={props.ref}
      class={styles.field}
      value={props.text}
      onChange={(event) => props.onText(event.currentTarget.value)}
    >
      <option value={SWITCH_ON}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_ON)}</option>
      <option value={SWITCH_OFF}>{t(I18N_KEYS.SETTINGS_EFFECTIVE_OFF)}</option>
    </select>
  ) : (
    <input
      id={SETTING_DIALOG_FIELD_ID}
      ref={props.ref}
      class={styles.field}
      type="text"
      inputMode="decimal"
      spellcheck={false}
      value={props.text}
      onInput={(event) => props.onText(event.currentTarget.value)}
      onKeyDown={(event) => event.key === SETTING_SUBMIT_KEY && props.onSubmit()}
    />
  );
}

/** The state of the dialog: the text of the field, the failure shown, and the two ways to save. */
function dialogState(setting: () => EffectiveSetting, onClose: () => void) {
  const kind = () => kindOf(setting().default);
  const kindText = () => t(KIND_TEXTS[kind()]);
  const [text, setText] = createSignal(setting().pending ?? setting().value);
  const [failure, setFailure] = createSignal<string | null>(null);
  const run = async (value: number | boolean | null) => {
    try {
      await saveSettingValue(setting().key, value);
      onClose();
    } catch (error) {
      setFailure(failureText(error, kindText()));
    }
  };
  const save = () => {
    const value = parseEntry(kind(), text());
    return value === null
      ? setFailure(t(I18N_KEYS.SETTINGS_SETTING_INVALID, { kind: kindText() }))
      : void run(value);
  };
  return { kind, kindText, text, setText, failure, run, save };
}

/** The field with its label and the hint: the kind of value and the default. */
function FieldRow(props: {
  setting: EffectiveSetting;
  state: ReturnType<typeof dialogState>;
  field: (element: HTMLInputElement | HTMLSelectElement) => void;
}) {
  const state = () => props.state;
  return (
    <div class={styles.row}>
      <label class={styles.label} for={SETTING_DIALOG_FIELD_ID}>
        {t(I18N_KEYS.SETTINGS_SETTING_NEW_VALUE)}
      </label>
      <Field
        switchKind={state().kind() === VALUE_KIND.switch}
        text={state().text()}
        onText={state().setText}
        onSubmit={state().save}
        ref={props.field}
      />
      <span class={styles.hint}>
        {state().kindText()}
        {HINT_SEPARATOR}
        {t(I18N_KEYS.SETTINGS_SETTING_DEFAULT_HINT, {
          value: displayValue({ ...props.setting, value: props.setting.default }),
        })}
      </span>
    </div>
  );
}

/** The three buttons: cancel, back to the default (only for a value saved in the app) and save. */
function Actions(props: {
  setting: EffectiveSetting;
  state: ReturnType<typeof dialogState>;
  onClose: () => void;
}) {
  const saved = () => props.setting.origin === ORIGIN.ui || props.setting.pending !== null;
  return (
    <div class={styles.actions}>
      <button type="button" class={styles.secondary} onClick={props.onClose}>
        {t(I18N_KEYS.SETTINGS_SETTING_CANCEL)}
      </button>
      <button
        type="button"
        class={styles.secondary}
        disabled={!saved()}
        title={saved() ? undefined : t(I18N_KEYS.SETTINGS_SETTING_NOTHING_SAVED)}
        onClick={() => void props.state.run(null)}
      >
        {t(I18N_KEYS.SETTINGS_SETTING_TO_DEFAULT)}
      </button>
      <button type="button" class={styles.primary} onClick={props.state.save}>
        {t(I18N_KEYS.SETTINGS_SETTING_SAVE)}
      </button>
    </div>
  );
}

/** The dialog "Change value" (mock 45 variant A, ADR 0118): the key, a field for the new value,
 * from when it counts, and the buttons to cancel, go back to the default and save. Enter in the
 * field saves, Esc closes the dialog. */
export function SettingDialog(props: { setting: EffectiveSetting; onClose: () => void }) {
  let dialog: HTMLDivElement | undefined;
  let field: HTMLInputElement | HTMLSelectElement | undefined;
  const state = dialogState(() => props.setting, props.onClose);
  useLayer(() => true);
  useKeydown((event) => event.key === SETTING_CANCEL_KEY && props.onClose());
  useDialogFocus(
    () => dialog,
    () => field,
  );
  return (
    <Portal>
      <div class={styles.scrim} onClick={props.onClose} />
      <div
        ref={dialog}
        class={styles.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby={SETTING_DIALOG_TITLE_ID}
      >
        <h2 id={SETTING_DIALOG_TITLE_ID} class={styles.title}>
          {t(I18N_KEYS.SETTINGS_SETTING_TITLE)}
        </h2>
        <code class={styles.key}>{props.setting.key}</code>
        <FieldRow setting={props.setting} state={state} field={(element) => (field = element)} />
        <Applies start={props.setting.edit === EDIT.start} />
        <Show when={state.failure()}>
          {(message) => (
            <p class={styles.error} role="alert">
              {message()}
            </p>
          )}
        </Show>
        <Actions setting={props.setting} state={state} onClose={props.onClose} />
      </div>
    </Portal>
  );
}
