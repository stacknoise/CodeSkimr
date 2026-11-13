import { Match, Switch } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { KIND, OPEN_COMMAND, REASON } from "../../shared/session/consts";
import { UNREACHABLE_CODE } from "./consts";
import type { SessionState } from "../../shared/session/session";
import styles from "./ConnectionScreen.module.css";

/** What the UI shows while it has no valid token: it explains why and names the command that
 * opens CodeSkimr with a new launch code (ADR 0028). */
export function ConnectionScreen(props: { state: SessionState }) {
  return (
    <section class={styles.card}>
      <h1>{t(I18N_KEYS.SESSION_REOPEN_TITLE)}</h1>
      <Switch>
        <Match when={props.state.kind === KIND.reopen && props.state.reason === REASON.codeInvalid}>
          <p class={styles.problem}>{t(I18N_KEYS.SESSION_CODE_USED)}</p>
        </Match>
        <Match when={props.state.kind === KIND.reopen && props.state.reason === REASON.unreachable}>
          <p class={styles.problem}>{errorText(UNREACHABLE_CODE)}</p>
        </Match>
      </Switch>
      <p>{t(I18N_KEYS.SESSION_REOPEN_TEXT)}</p>
      <code class={styles.command}>{OPEN_COMMAND}</code>
    </section>
  );
}
