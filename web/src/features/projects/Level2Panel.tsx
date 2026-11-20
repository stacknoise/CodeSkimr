import { createSignal, onMount, Show } from "solid-js";
import { currentLanguage, t, type MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  COPIED_MS,
  ESCAPE_KEY,
  LEVEL2_COOLDOWN_SECONDS,
  LEVEL2_CONFIG_SNIPPET,
  LEVEL2_ERROR,
  LEVEL2_PANEL_ID_PREFIX,
  LEVEL2_PHASE,
  LEVEL2_STATE,
} from "./consts";
import { failureText } from "./failure";
import { clock } from "./format";
import {
  askRevoke,
  cancelRevoke,
  cancelWaiting,
  dismissRevoked,
  escapeWaiting,
  focusSwitch,
  level2Of,
  loadLevel2,
  switchOff,
  switchOn,
  type Level2View,
} from "./level2";
import { rescanProject } from "./projects";
import { isChecked, stateLook } from "./Level2Switch";
import { Pill } from "../../shared/ui/Pill";
import type { ProjectRow } from "./types";
import styles from "./Level2Panel.module.css";

/** The command of the terminal, with a button that copies it (the fallback of ADR 0071,
 * decision 6). */
function CommandBox(props: { row: ProjectRow; command: string }) {
  const [copied, setCopied] = createSignal(false);
  const copy = () => {
    void navigator.clipboard
      .writeText(props.command)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), COPIED_MS);
      })
      .catch(() => undefined);
  };
  return (
    <div class={styles.command}>
      <code
        class={styles.code}
        role="group"
        aria-label={t(I18N_KEYS.PROJECTS_LEVEL2_COMMAND_LABEL, { name: props.row.name })}
      >
        {props.command}
      </code>
      <button type="button" class={styles.button} onClick={copy}>
        {copied() ? t(I18N_KEYS.PROJECTS_LEVEL2_COPIED) : t(I18N_KEYS.PROJECTS_LEVEL2_COPY)}
      </button>
    </div>
  );
}

/** What the state means, and what to do about it. */
function Explain(props: { view: Level2View }) {
  const status = () => props.view.status;
  return (
    <>
      <Show
        when={status()}
        fallback={<p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_EXPLAIN_UNKNOWN)}</p>}
      >
        {(known) => (
          <>
            <Show when={known().state === LEVEL2_STATE.on}>
              <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_EXPLAIN_ON)}</p>
            </Show>
            <Show when={known().state === LEVEL2_STATE.off}>
              <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_EXPLAIN_OFF)}</p>
            </Show>
            <Show when={known().state === LEVEL2_STATE.locked}>
              <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_EXPLAIN_LOCKED)}</p>
            </Show>
            <Show when={known().state === LEVEL2_STATE.notEnabled}>
              <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_EXPLAIN_NOT_ENABLED)}</p>
              <pre class={styles.snippet}>{LEVEL2_CONFIG_SNIPPET}</pre>
              <p class={styles.path}>{t(I18N_KEYS.PROJECTS_LEVEL2_CONFIG_FILE)}</p>
            </Show>
          </>
        )}
      </Show>
    </>
  );
}

/** The question before switching off: one click on "Yes", Escape or "Cancel" to leave. */
function RevokeAsk(props: { row: ProjectRow }) {
  let yes: HTMLButtonElement | undefined;
  onMount(() => yes?.focus());
  const leave = () => {
    cancelRevoke(props.row.id);
    focusSwitch(props.row.id);
  };
  return (
    <div
      class={styles.ask}
      role="group"
      aria-label={t(I18N_KEYS.PROJECTS_LEVEL2_REVOKE_ASK, { name: props.row.name })}
      onKeyDown={(event) => {
        if (event.key === ESCAPE_KEY) {
          event.stopPropagation();
          leave();
        }
      }}
    >
      <strong>{t(I18N_KEYS.PROJECTS_LEVEL2_REVOKE_ASK, { name: props.row.name })}</strong>
      <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_REVOKE_TEXT)}</p>
      <div class={styles.buttons}>
        <button
          type="button"
          class={styles.primary}
          ref={(element) => (yes = element)}
          onClick={() => {
            void switchOff(props.row.id).then(() => focusSwitch(props.row.id));
          }}
        >
          {t(I18N_KEYS.PROJECTS_LEVEL2_REVOKE_YES)}
        </button>
        <button type="button" class={styles.button} onClick={leave}>
          {t(I18N_KEYS.PROJECTS_LEVEL2_REVOKE_NO)}
        </button>
      </div>
    </div>
  );
}

/** What was started: the console window that is open, or the command that waits in the terminal. */
function WaitingWhat(props: { row: ProjectRow; view: Level2View }) {
  const command = () => props.view.command;
  return (
    <>
      <strong>
        {command() === null
          ? t(I18N_KEYS.PROJECTS_LEVEL2_WAITING_TITLE)
          : t(I18N_KEYS.PROJECTS_LEVEL2_COPY_TITLE)}
      </strong>
      <Show
        when={command()}
        fallback={
          <p class={styles.text}>
            {t(I18N_KEYS.PROJECTS_LEVEL2_WAITING_TEXT, {
              time: clock(props.view.openedAt ?? Date.now(), currentLanguage()),
            })}
          </p>
        }
      >
        {(text) => (
          <>
            <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_COPY_TEXT)}</p>
            <CommandBox row={props.row} command={text()} />
          </>
        )}
      </Show>
    </>
  );
}

/** The window is open, or the command waits in the terminal of the user. The page cannot see that
 * the window was closed, so "Cancel waiting" (or Escape) is the way out before the time is up. */
function Waiting(props: { row: ProjectRow; view: Level2View }) {
  return (
    <div class={styles.notice}>
      <WaitingWhat row={props.row} view={props.view} />
      <p class={styles.waiting}>{t(I18N_KEYS.PROJECTS_LEVEL2_WAITING_STATE)}</p>
      <p class={styles.waiting}>
        {t(I18N_KEYS.PROJECTS_LEVEL2_WAITING_PAUSE, { seconds: LEVEL2_COOLDOWN_SECONDS })}
      </p>
      <div class={styles.buttons}>
        <button type="button" class={styles.button} onClick={() => cancelWaiting(props.row.id)}>
          {t(I18N_KEYS.PROJECTS_LEVEL2_WAITING_CANCEL)}
        </button>
      </div>
    </div>
  );
}

/** The notice that follows a switch-off: what stays, and the way to a new scan. */
function Revoked(props: { row: ProjectRow }) {
  return (
    <div class={styles.notice}>
      <p class={styles.text}>{t(I18N_KEYS.PROJECTS_LEVEL2_REVOKED)}</p>
      <div class={styles.buttons}>
        <button
          type="button"
          class={styles.button}
          onClick={() => {
            dismissRevoked(props.row.id);
            void rescanProject(props.row.id);
            focusSwitch(props.row.id);
          }}
        >
          {t(I18N_KEYS.PROJECTS_LEVEL2_RESCAN)}
        </button>
      </div>
    </div>
  );
}

/** A failure in words; after a pause of the server it says how long to wait. */
function Failure(props: { view: Level2View }) {
  return (
    <Show when={props.view.failure} keyed>
      {(failure) => (
        <div class={styles.error} role="alert">
          <p class={styles.text}>{failureText(failure)}</p>
          <Show when={failure.code === LEVEL2_ERROR.rateLimited}>
            <p class={styles.text}>
              {t(I18N_KEYS.PROJECTS_LEVEL2_RETRY_WAIT, {
                seconds: failure.retryAfter ?? LEVEL2_COOLDOWN_SECONDS,
              })}
            </p>
          </Show>
        </div>
      )}
    </Show>
  );
}

/** The action that the state offers: switch on, or check again if the state is unknown. */
function Actions(props: { row: ProjectRow; view: Level2View }) {
  const state = () => props.view.status?.state;
  const idle = () =>
    props.view.phase === LEVEL2_PHASE.idle ||
    props.view.phase === LEVEL2_PHASE.timedOut ||
    props.view.phase === LEVEL2_PHASE.cancelled ||
    props.view.phase === LEVEL2_PHASE.revoked;
  return (
    <div class={styles.buttons}>
      <Show when={state() === LEVEL2_STATE.off && idle()}>
        <button
          type="button"
          class={styles.primary}
          onClick={() => {
            void switchOn(props.row.id);
            focusSwitch(props.row.id);
          }}
        >
          {t(I18N_KEYS.PROJECTS_LEVEL2_SWITCH_ON)}
        </button>
      </Show>
      <Show when={isChecked(props.view) && props.view.phase === LEVEL2_PHASE.idle}>
        <button type="button" class={styles.button} onClick={() => askRevoke(props.row.id)}>
          {t(I18N_KEYS.PROJECTS_LEVEL2_SWITCH_OFF)}
        </button>
      </Show>
      <Show when={state() === undefined || state() === LEVEL2_STATE.notEnabled}>
        <button type="button" class={styles.button} onClick={() => void loadLevel2(props.row.id)}>
          {t(I18N_KEYS.PROJECTS_LEVEL2_CHECK_AGAIN)}
        </button>
      </Show>
    </div>
  );
}

/** The phases in which the explanation and the actions of the state are shown. */
const SETTLED: readonly string[] = [
  LEVEL2_PHASE.idle,
  LEVEL2_PHASE.timedOut,
  LEVEL2_PHASE.cancelled,
];

/** The one-line notices of the phases that need nothing but words. */
const NOTE: Readonly<Record<string, MessageKey | undefined>> = {
  [LEVEL2_PHASE.opening]: I18N_KEYS.PROJECTS_LEVEL2_OPENING,
  [LEVEL2_PHASE.revoking]: I18N_KEYS.PROJECTS_LEVEL2_REVOKING,
  [LEVEL2_PHASE.timedOut]: I18N_KEYS.PROJECTS_LEVEL2_TIMED_OUT,
  [LEVEL2_PHASE.cancelled]: I18N_KEYS.PROJECTS_LEVEL2_CANCELLED,
};

/** The level 2 section of the selected project: the state in words, what the next click does, the
 * waiting for the console window, the command to copy where no window can open, and errors
 * (ADR 0071). Announced to screen readers as a status region. */
export function Level2Panel(props: { row: ProjectRow }) {
  const view = () => level2Of(props.row.id);
  const phase = () => view().phase;
  return (
    <section
      class={styles.panel}
      id={`${LEVEL2_PANEL_ID_PREFIX}${props.row.id}`}
      aria-label={t(I18N_KEYS.PROJECTS_LEVEL2_TITLE, { name: props.row.name })}
      onKeyDown={(event) => escapeWaiting(props.row.id, event)}
    >
      <header class={styles.header}>
        <h2>{t(I18N_KEYS.PROJECTS_LEVEL2_TITLE, { name: props.row.name })}</h2>
        <Pill tone={stateLook(view()).tone}>{t(stateLook(view()).text)}</Pill>
      </header>
      <div aria-live="polite" class={styles.live}>
        <Show when={NOTE[phase()]} keyed>
          {(note) => <p class={styles.text}>{t(note)}</p>}
        </Show>
        <Show when={phase() === LEVEL2_PHASE.waiting}>
          <Waiting row={props.row} view={view()} />
        </Show>
        <Show when={phase() === LEVEL2_PHASE.revoked}>
          <Revoked row={props.row} />
        </Show>
      </div>
      <Failure view={view()} />
      <Show when={phase() === LEVEL2_PHASE.askRevoke}>
        <RevokeAsk row={props.row} />
      </Show>
      <Show when={SETTLED.includes(phase())}>
        <Explain view={view()} />
        <Actions row={props.row} view={view()} />
      </Show>
    </section>
  );
}
