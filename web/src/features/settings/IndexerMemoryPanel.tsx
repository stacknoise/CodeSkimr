import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  GENERAL_MEMORY_KEY,
  INDEXER_MEMORY_ID_PREFIX,
  INDEXER_MEMORY_MAX,
  INDEXER_MEMORY_PREFIX,
  ORIGIN,
} from "./consts";
import { effectiveSettings, loadEffectiveSettings, type EffectiveSetting } from "./effective";
import { loadUiSettings, saveIndexerMemory, uiSettings } from "./indexerMemory";
import styles from "./IndexerMemoryPanel.module.css";

/** What a row says below its buttons after the last action. */
const NOTE = { none: "none", saved: "saved", failed: "failed", invalid: "invalid" } as const;
type Note = (typeof NOTE)[keyof typeof NOTE];

const WHOLE_NUMBER = /^\d+$/;

/** The megabytes in a text, or `null` if the text is not a whole number the server accepts. */
export function parseMegabytes(text: string): number | null {
  const trimmed = text.trim();
  if (!WHOLE_NUMBER.test(trimmed)) {
    return null;
  }
  const value = Number(trimmed);
  return value <= INDEXER_MEMORY_MAX ? value : null;
}

/** The text that says what the indexer does at `0`, or the empty text for any other value. */
function followsText(draft: string, general: string): string {
  return draft.trim() === "0"
    ? t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_FOLLOWS, { limit: general })
    : "";
}

/** The words after the buttons of a row: what 0 means, the flag, and the result of the last
 * action. */
function Notes(props: { note: Note; byFlag: boolean; draft: string; general: string }) {
  return (
    <>
      <span class={styles.muted}>{followsText(props.draft, props.general)}</span>
      <Show when={props.byFlag}>
        <span class={styles.muted}>{t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_BY_FLAG)}</span>
      </Show>
      <Show when={props.note === NOTE.saved}>
        <span role="status" class={styles.muted}>
          {t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_SAVED)}
        </span>
      </Show>
      <Show when={props.note === NOTE.failed}>
        <span role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_FAILED)}
        </span>
      </Show>
      <Show when={props.note === NOTE.invalid}>
        <span role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_INVALID, { max: INDEXER_MEMORY_MAX })}
        </span>
      </Show>
    </>
  );
}

/** The text of the field, the last result and the two actions of one indexer. */
function rowState(name: () => string, shown: () => string) {
  const [edited, setEdited] = createSignal<string | null>(null);
  const [note, setNote] = createSignal<Note>(NOTE.none);
  const run = async (megabytes: number | null) => {
    try {
      await saveIndexerMemory(name(), megabytes);
      setEdited(null);
      setNote(NOTE.saved);
    } catch {
      setNote(NOTE.failed);
    }
  };
  const draft = () => edited() ?? shown();
  const save = () => {
    const megabytes = parseMegabytes(draft());
    if (megabytes === null) {
      setNote(NOTE.invalid);
      return;
    }
    void run(megabytes);
  };
  const type = (text: string) => {
    setEdited(text);
    setNote(NOTE.none);
  };
  return { draft, note, save, type, forget: () => void run(null) };
}

/** The two buttons of a row: save, and, if a value is saved, go back to the default. */
function Buttons(props: { saved: boolean; byFlag: boolean; save: () => void; forget: () => void }) {
  return (
    <>
      <button
        type="button"
        class={`${styles.button} ${styles.primary}`}
        disabled={props.byFlag}
        onClick={props.save}
      >
        {t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_SAVE)}
      </button>
      <Show when={props.saved}>
        <button type="button" class={styles.button} disabled={props.byFlag} onClick={props.forget}>
          {t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_RESET)}
        </button>
      </Show>
    </>
  );
}

function Row(props: { name: string; setting: EffectiveSetting | undefined; general: string }) {
  const saved = () => uiSettings()?.indexer_memory_mb[props.name];
  const state = rowState(
    () => props.name,
    () => String(saved() ?? props.setting?.value ?? ""),
  );
  const byFlag = () => props.setting?.origin === ORIGIN.cli;
  const id = () => `${INDEXER_MEMORY_ID_PREFIX}${props.name}`;
  return (
    <li class={styles.row}>
      <label class={styles.name} for={id()}>
        {props.name}
      </label>
      <input
        id={id()}
        class={styles.field}
        type="text"
        inputmode="numeric"
        aria-label={t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_FIELD, { name: props.name })}
        value={state.draft()}
        disabled={byFlag()}
        onInput={(event) => state.type(event.currentTarget.value)}
      />
      <div class={styles.actions}>
        <Buttons
          saved={saved() !== undefined}
          byFlag={byFlag()}
          save={state.save}
          forget={state.forget}
        />
        <Notes
          note={state.note()}
          byFlag={byFlag()}
          draft={state.draft()}
          general={props.general}
        />
      </div>
    </li>
  );
}

/** The memory limit per indexer of level 2 (ADR 0100, ADR 0103). The values come from the
 * effective settings, the saved ones from the server; a saved value counts from the next start. */
export function IndexerMemoryPanel() {
  onMount(() => {
    void loadEffectiveSettings();
    void loadUiSettings();
  });
  const names = createMemo(
    () =>
      effectiveSettings()
        ?.settings.filter((setting) => setting.key.startsWith(INDEXER_MEMORY_PREFIX))
        .map((setting) => setting.key.slice(INDEXER_MEMORY_PREFIX.length)) ?? [],
  );
  const settingOf = (name: string) =>
    effectiveSettings()?.settings.find(
      (setting) => setting.key === `${INDEXER_MEMORY_PREFIX}${name}`,
    );
  const general = () =>
    effectiveSettings()?.settings.find((setting) => setting.key === GENERAL_MEMORY_KEY)?.value ??
    "";
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_TITLE)}>
      <div>
        <h2>{t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_TITLE)}</h2>
        <p class={styles.note}>{t(I18N_KEYS.SETTINGS_INDEXER_MEMORY_NOTE)}</p>
      </div>
      <ul class={styles.rows}>
        <For each={names()}>
          {(name) => <Row name={name} setting={settingOf(name)} general={general()} />}
        </For>
      </ul>
    </section>
  );
}
