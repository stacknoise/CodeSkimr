import { createMemo, For, Match, Show, Switch } from "solid-js";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { showUsages } from "../../shared/navigation/navigation";
import { PILL_SEPARATOR } from "./consts";
import { sourceWidth } from "./layout";
import { lineRange } from "./format";
import { type Segment, segments } from "./segments";
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

/** Alt+click on the name of a symbol in the code opens its usages (blueprint 15.4). */
function onNameClick(event: MouseEvent): void {
  const target = event.target;
  const symbol = target instanceof HTMLElement ? target.dataset.symbol : undefined;
  if (event.altKey && symbol !== undefined) {
    showUsages(Number(symbol));
    event.preventDefault();
  }
}

/** The class of the colour of a piece; plain code has none (`styles.plain` is no rule). */
function colourOf(piece: Segment): string | undefined {
  return piece.tokenClass === null ? undefined : styles[piece.tokenClass];
}

/** The line numbers and the code side by side. The line numbers are plain text; the code is
 * coloured by its syntax, and the names of symbols are small elements of their own. */
function Code(props: { info: SourceInfo }) {
  const numbers = createMemo(() =>
    props.info.lines.map((_, index) => props.info.first_line + index).join("\n"),
  );
  const pieces = createMemo(() =>
    segments(props.info.lines, props.info.spans ?? [], props.info.tokens ?? []),
  );
  return (
    <div class={styles.code}>
      <pre class={styles.gutter} aria-hidden="true">
        {numbers()}
      </pre>
      <pre class={styles.text} onClick={onNameClick}>
        <code>
          <For each={pieces()}>
            {(piece) => (
              <Show
                when={piece.symbol !== null}
                fallback={
                  <Show when={colourOf(piece)} fallback={piece.text}>
                    {(colour) => <span class={colour()}>{piece.text}</span>}
                  </Show>
                }
              >
                <span
                  class={[styles.name, colourOf(piece)].filter(Boolean).join(" ")}
                  data-symbol={piece.symbol}
                  title={t(I18N_KEYS.EXPLORER_SOURCE_NAME_HINT)}
                >
                  {piece.text}
                </span>
              </Show>
            )}
          </For>
        </code>
      </pre>
    </div>
  );
}

function Lines(props: { info: SourceInfo }) {
  return (
    <Switch>
      <Match when={props.info.excluded}>
        <Note text={t(I18N_KEYS.EXPLORER_SOURCE_EXCLUDED)} />
      </Match>
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

/** What the right column shows: a hint, a failure, the wait, or the source. */
function Content(props: { shown: boolean }) {
  return (
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
  );
}

/** The right column of the explorer: file, position and code of the selected type or member
 * (blueprint B.2, mock `Explorer.dc.html`). With a member selected it sits under the header in the
 * middle column and takes all the room there (`wide`). */
export function SourcePane(props: { shown: boolean; wide: boolean }) {
  return (
    <section
      class={cx(styles.pane, props.wide && styles.inColumn)}
      style={props.wide ? undefined : { "flex-basis": `${sourceWidth.size()}px` }}
      aria-label={t(I18N_KEYS.EXPLORER_SOURCE_LABEL)}
    >
      <Content shown={props.shown} />
    </section>
  );
}
