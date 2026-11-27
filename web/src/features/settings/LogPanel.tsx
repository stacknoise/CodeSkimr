import { For, Show, createSignal, onCleanup, onMount } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { size } from "../projects/format";
import {
  COPIED_MS,
  LOG_LEVEL_TEXTS,
  LOG_REFRESH_MS,
  MINUTES_PER_HOUR,
  SECONDS_PER_MINUTE,
} from "./consts";
import {
  chooseLogLevel,
  clearLog,
  loadLog,
  logLines,
  logStatus,
  logText,
  openLogFolder,
  type LogStatus,
} from "./log";
import styles from "./LogPanel.module.css";

/** What the buttons say below them after the last action. */
const NOTE = {
  none: "none",
  copied: "copied",
  copyFailed: "copy-failed",
  cleared: "cleared",
} as const;
type Note = (typeof NOTE)[keyof typeof NOTE];

/** The words of a level; a level the texts do not know is shown as it is. */
function levelText(level: string): string {
  const key = LOG_LEVEL_TEXTS[level];
  return key === undefined ? level : t(key);
}

/** The time that is left, as hours and minutes. */
function leftText(seconds: number): string {
  const minutes = Math.ceil(seconds / SECONDS_PER_MINUTE);
  return t(I18N_KEYS.SETTINGS_LOG_RESETS, {
    hours: Math.floor(minutes / MINUTES_PER_HOUR),
    minutes: minutes % MINUTES_PER_HOUR,
  });
}

/** The row of levels: one is chosen, the others change the level at once. */
function Levels(props: { status: LogStatus; onFailure: () => void }) {
  const choose = (level: string) => {
    void chooseLogLevel(level).catch(props.onFailure);
  };
  return (
    <div class={styles.levels} role="radiogroup" aria-label={t(I18N_KEYS.SETTINGS_LOG_LEVEL)}>
      <For each={props.status.levels}>
        {(level) => (
          <button
            type="button"
            role="radio"
            class={styles.level}
            aria-checked={props.status.level === level}
            disabled={!props.status.adjustable}
            onClick={() => choose(level)}
          >
            {levelText(level)}
          </button>
        )}
      </For>
    </div>
  );
}

/** The word on a button for a moment after its action, or the failure of the copy. */
function actionNotes() {
  const [note, setNote] = createSignal<Note>(NOTE.none);
  const hold = (next: Note) => {
    setNote(next);
    setTimeout(() => setNote(NOTE.none), COPIED_MS);
  };
  const copy = () => {
    void logText()
      .then((text) => navigator.clipboard.writeText(text))
      .then(() => hold(NOTE.copied))
      .catch(() => hold(NOTE.copyFailed));
  };
  return { note, hold, copy };
}

/** The buttons under the file: copy, open the folder, empty. */
function Actions(props: { status: LogStatus; onFailure: () => void }) {
  const { note, hold, copy } = actionNotes();
  const clear = () => {
    void clearLog()
      .then(() => hold(NOTE.cleared))
      .catch(props.onFailure);
  };
  return (
    <div class={styles.row}>
      <button type="button" class={`${styles.button} ${styles.primary}`} onClick={copy}>
        {note() === NOTE.copied ? t(I18N_KEYS.SETTINGS_LOG_COPIED) : t(I18N_KEYS.SETTINGS_LOG_COPY)}
      </button>
      <button
        type="button"
        class={styles.button}
        disabled={!props.status.can_open_folder}
        onClick={() => void openLogFolder().catch(props.onFailure)}
      >
        {t(I18N_KEYS.SETTINGS_LOG_OPEN_FOLDER)}
      </button>
      <button type="button" class={styles.button} onClick={clear}>
        {note() === NOTE.cleared
          ? t(I18N_KEYS.SETTINGS_LOG_CLEARED)
          : t(I18N_KEYS.SETTINGS_LOG_CLEAR)}
      </button>
      <Show when={note() === NOTE.copyFailed}>
        <span role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_LOG_COPY_FAILED)}
        </span>
      </Show>
    </div>
  );
}

/** The file with its size and the limit of the rotation. */
function FileRow(props: { status: LogStatus }) {
  const language = currentLanguage;
  return (
    <div class={styles.row}>
      <span class={styles.label}>{t(I18N_KEYS.SETTINGS_LOG_FILE)}</span>
      <Show
        when={props.status.file}
        fallback={<span class={styles.muted}>{t(I18N_KEYS.SETTINGS_LOG_NO_FILE)}</span>}
      >
        {(file) => <span class={styles.mono}>{file()}</span>}
      </Show>
      <span class={styles.muted}>{size(props.status.size_bytes, language())}</span>
      <span class={styles.muted}>
        {t(I18N_KEYS.SETTINGS_LOG_LIMIT, {
          count: props.status.files_kept,
          size: size(props.status.max_file_bytes, language()),
        })}
      </span>
    </div>
  );
}

/** The level, what it means, the file, the buttons and the last lines of a loaded log. */
function LogBody(props: { status: LogStatus; onFailure: () => void }) {
  return (
    <>
      <span class={styles.label}>{t(I18N_KEYS.SETTINGS_LOG_LEVEL)}</span>
      <Levels status={props.status} onFailure={props.onFailure} />
      <p class={styles.muted}>
        {t(I18N_KEYS.SETTINGS_LOG_DEFAULT, { level: levelText(props.status.default_level) })}{" "}
        <Show when={props.status.resets_in_seconds}>
          {(seconds) => <strong>{leftText(seconds())}</strong>}
        </Show>
      </p>
      <Show when={!props.status.adjustable}>
        <p class={styles.muted}>{t(I18N_KEYS.SETTINGS_LOG_FIXED)}</p>
      </Show>
      <FileRow status={props.status} />
      <Actions status={props.status} onFailure={props.onFailure} />
      <span class={styles.label}>{t(I18N_KEYS.SETTINGS_LOG_LAST_LINES)}</span>
      <pre class={styles.lines} aria-live="off">
        {logLines().length > 0 ? logLines().join("\n") : t(I18N_KEYS.SETTINGS_LOG_NO_LINES)}
      </pre>
    </>
  );
}

/** The tab Diagnosis: the log level, the file and its last lines (ADR 0113, mock 42 variant A). */
export function LogPanel() {
  const [failed, setFailed] = createSignal(false);
  const refresh = () => {
    void loadLog()
      .then(() => setFailed(false))
      .catch(() => setFailed(true));
  };
  onMount(() => {
    refresh();
    const timer = setInterval(refresh, LOG_REFRESH_MS);
    onCleanup(() => clearInterval(timer));
  });
  return (
    <section class={styles.panel} aria-label={t(I18N_KEYS.SETTINGS_LOG_TITLE)}>
      <h2>{t(I18N_KEYS.SETTINGS_LOG_TITLE)}</h2>
      <p class={styles.note}>{t(I18N_KEYS.SETTINGS_LOG_NOTE)}</p>
      <Show when={failed()}>
        <p role="alert" class={styles.error}>
          {t(I18N_KEYS.SETTINGS_LOG_FAILED)}
        </p>
      </Show>
      <Show when={logStatus()}>
        {(status) => <LogBody status={status()} onFailure={() => setFailed(true)} />}
      </Show>
    </section>
  );
}
