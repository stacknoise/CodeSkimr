import { createMemo, Match, Show, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { PILL_SEPARATOR } from "./consts";
import { lineRange } from "./format";
import { sourceError, sourceInfo, sourceLoading } from "./source";
import type { SourceInfo } from "./types";
import styles from "./SourcePane.module.css";

function Note(props: { text: string; alert?: boolean }) {
  return (
    <p class={styles.note} role={props.alert ? "alert" : undefined}>
      {props.text}
    </p>
  );
}

/** The line numbers and the code side by side; both are plain text, so a long declaration costs
 * two nodes, not hundreds. */
function Code(props: { info: SourceInfo }) {
  const numbers = createMemo(() =>
    props.info.lines.map((_, index) => props.info.first_line + index).join("\n"),
  );
  return (
    <div class={styles.code}>
      <pre class={styles.gutter} aria-hidden="true">
        {numbers()}
      </pre>
      <pre class={styles.text}>
        <code>{props.info.lines.join("\n")}</code>
      </pre>
    </div>
  );
}

function Lines(props: { info: SourceInfo }) {
  return (
    <Switch>
      <Match when={!props.info.available}>
        <Note text={t(I18N_KEYS.EXPLORER_SOURCE_UNAVAILABLE)} />
      </Match>
      <Match when={props.info.lines.length === 0}>
        <Note text={t(I18N_KEYS.EXPLORER_SOURCE_EMPTY)} />
      </Match>
      <Match when={true}>
        <Code info={props.info} />
        <Show when={props.info.truncated}>
          <Note text={t(I18N_KEYS.EXPLORER_SOURCE_TRUNCATED, { count: props.info.lines.length })} />
        </Show>
      </Match>
    </Switch>
  );
}

/** The right column of the explorer: file, position and code of the selected type or member
 * (blueprint B.2, mock `Explorer.dc.html`). */
export function SourcePane(props: { shown: boolean }) {
  return (
    <section class={styles.pane} aria-label={t(I18N_KEYS.EXPLORER_SOURCE_LABEL)}>
      <Switch>
        <Match when={!props.shown}>
          <Note text={t(I18N_KEYS.EXPLORER_SOURCE_PICK)} />
        </Match>
        <Match when={sourceError()}>
          {(failure) => (
            <Note
              text={
                failure().code === null
                  ? t(I18N_KEYS.EXPLORER_SOURCE_FAILED)
                  : errorText(failure().code as ErrorCode)
              }
              alert
            />
          )}
        </Match>
        <Match when={sourceLoading()}>
          <Note text={t(I18N_KEYS.EXPLORER_SOURCE_LOADING)} />
        </Match>
        <Match when={sourceInfo()}>
          {(info) => (
            <>
              <div class={styles.head}>
                <span class={styles.path}>{info().path}</span>
                <span>
                  {PILL_SEPARATOR}
                  {lineRange(info().first_line, info().last_line)}
                </span>
              </div>
              <Lines info={info()} />
            </>
          )}
        </Match>
      </Switch>
    </section>
  );
}
