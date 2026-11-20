import { For, Match, Switch } from "solid-js";
import { PIECE } from "./consts";
import { parseMarkup } from "./markup";
import styles from "./RichText.module.css";

const NEW_TAB = "_blank";
const LINK_REL = "noopener noreferrer";

/** A text of a note or a task in the small Markdown subset, drawn as elements and text nodes, never
 * as HTML. A link opens in a new tab and only `http` and `https` addresses are links at all. */
export function RichText(props: { text: string }) {
  return (
    <span class={styles.text}>
      <For each={parseMarkup(props.text)}>
        {(piece) => (
          <Switch fallback={piece.text}>
            <Match when={piece.kind === PIECE.bold}>
              <strong>{piece.text}</strong>
            </Match>
            <Match when={piece.kind === PIECE.italic}>
              <em>{piece.text}</em>
            </Match>
            <Match when={piece.kind === PIECE.code}>
              <code class={styles.code}>{piece.text}</code>
            </Match>
            <Match when={piece.kind === PIECE.link}>
              <a href={piece.href} target={NEW_TAB} rel={LINK_REL}>
                {piece.text}
              </a>
            </Match>
          </Switch>
        )}
      </For>
    </span>
  );
}
