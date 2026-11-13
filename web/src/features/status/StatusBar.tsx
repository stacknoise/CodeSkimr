import { createEffect, createMemo, For, Match, onCleanup, Show, Switch } from "solid-js";
import { currentLanguage, isLanguage, setLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { lastQueryMs } from "../../shared/api/timings";
import { KIND } from "../../shared/session/consts";
import { sessionState } from "../../shared/session/session";
import { cx } from "../../shared/ui/cx";
import { LANGUAGES } from "../../shared/i18n/consts";
import { projects, projectsLoaded, refreshProjects } from "../projects/projects";
import { STATUS_DOT } from "../../app/consts";
import { COUNT_SEPARATOR, LANGUAGE_LABELS, STATUS_POLL_MS } from "./consts";
import styles from "./StatusBar.module.css";
import { refreshStatus, statusInfo } from "./status";

function Connection() {
  return (
    <Switch>
      <Match when={sessionState().kind === KIND.connected}>
        <span class={styles.ok}>
          {STATUS_DOT} {t(I18N_KEYS.STATUS_CONNECTED)}
        </span>
      </Match>
      <Match when={sessionState().kind === KIND.connecting}>
        <span>{t(I18N_KEYS.STATUS_CONNECTING)}</span>
      </Match>
      <Match when={true}>
        <span>{t(I18N_KEYS.STATUS_DISCONNECTED)}</span>
      </Match>
    </Switch>
  );
}

function Counts() {
  const types = createMemo(() => projects().reduce((sum, project) => sum + project.types, 0));
  return (
    <Show when={projectsLoaded()}>
      <span>
        {t(I18N_KEYS.APP_STATUS_PROJECTS, { count: projects().length })}
        {COUNT_SEPARATOR}
        {t(I18N_KEYS.APP_STATUS_TYPES, { count: types() })}
      </span>
    </Show>
  );
}

function Measurements() {
  return (
    <>
      <Show when={statusInfo()?.startup_ms ?? null}>
        {(ms) => <span>{t(I18N_KEYS.APP_STARTUP_TIME, { ms: ms() })}</span>}
      </Show>
      <Show when={lastQueryMs() !== null}>
        <span>{t(I18N_KEYS.APP_QUERY_TIME, { ms: Math.round(lastQueryMs() ?? 0) })}</span>
      </Show>
      <Show when={statusInfo()?.memory_mb ?? null}>
        {(mb) => <span>{t(I18N_KEYS.APP_MEMORY_USAGE, { mb: mb() })}</span>}
      </Show>
    </>
  );
}

/** Switches the language of all texts at once; the choice is stored like the one in the
 * settings. */
function LanguageSwitch() {
  return (
    <div class={styles.language} role="group" aria-label={t(I18N_KEYS.APP_LANGUAGE)}>
      <For each={LANGUAGES}>
        {(language) => (
          <button
            type="button"
            class={cx(styles.choice, currentLanguage() === language && styles.on)}
            aria-pressed={currentLanguage() === language}
            onClick={() => isLanguage(language) && setLanguage(language)}
          >
            {LANGUAGE_LABELS[language]}
          </button>
        )}
      </For>
    </div>
  );
}

/** The status bar of every screen: connection, counts, start time, memory and the language
 * switch. The numbers are measured, not decoration (blueprint 15.5). */
export function StatusBar() {
  createEffect(() => {
    if (sessionState().kind !== KIND.connected) {
      return;
    }
    void refreshStatus();
    if (!projectsLoaded()) {
      void refreshProjects();
    }
    const timer = setInterval(() => void refreshStatus(), STATUS_POLL_MS);
    onCleanup(() => clearInterval(timer));
  });
  return (
    <footer class={styles.status}>
      <Connection />
      <Counts />
      <Measurements />
      <LanguageSwitch />
    </footer>
  );
}
