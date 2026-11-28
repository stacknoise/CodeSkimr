import { For, Show } from "solid-js";
import { createHighlight } from "./highlight";
import { linesOf } from "./parts";
import styles from "./Syntax.module.css";
import type { Part } from "./types";
import { LINE_END } from "./consts";

/** The class of the colour of a part; plain code has none. */
export function syntaxClass(part: Part): string | undefined {
  return part.syntax === null ? undefined : styles[part.syntax];
}

/** A piece of code, coloured with the syntax of the language of `path` (the same colours as the
 * source column of the explorer). Whitespace and indentation stay as they are. Until the colours
 * are there, and for a language without colours, the code is shown plain. Every place that shows
 * code outside the source column uses this one. */
export function Code(props: {
  path: string;
  text: string;
  class?: string | undefined;
  title?: string | undefined;
}) {
  const tokens = createHighlight(
    () => props.path,
    () => props.text,
  );
  const lines = () => linesOf(props.text, tokens());
  return (
    <code class={props.class} title={props.title}>
      <For each={lines()}>
        {(parts, index) => (
          <>
            <For each={parts}>
              {(part) => (
                <Show when={syntaxClass(part)} fallback={part.text}>
                  {(colour) => <span class={colour()}>{part.text}</span>}
                </Show>
              )}
            </For>
            <Show when={index() < lines().length - 1}>{LINE_END}</Show>
          </>
        )}
      </For>
    </code>
  );
}
