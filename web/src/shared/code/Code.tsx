import { For, Show } from "solid-js";
import { createHighlight } from "./highlight";
import { linesOf } from "./parts";
import styles from "./Syntax.module.css";
import type { MarkRange, Part } from "./types";
import { LINE_END } from "./consts";

/** The class of the colour of a part; plain code has none. */
export function syntaxClass(part: Part): string | undefined {
  return part.syntax === null ? undefined : styles[part.syntax];
}

/** One part: coloured if it has a class, and inside a `mark` element if it lies in the marked stretch. */
function PartView(props: { part: Part; markClass?: string | undefined }) {
  const inner = () => (
    <Show when={syntaxClass(props.part)} fallback={props.part.text}>
      {(colour) => <span class={colour()}>{props.part.text}</span>}
    </Show>
  );
  return (
    <Show when={props.part.marked === true} fallback={inner()}>
      <mark class={props.markClass}>{inner()}</mark>
    </Show>
  );
}

/** A piece of code, coloured with the syntax of the language of `path` (the same colours as the
 * source column of the explorer). Whitespace and indentation stay as they are. Until the colours
 * are there, and for a language without colours, the code is shown plain. A stretch of the first
 * line can be `mark`ed (the hit list marks the symbol that is used). Every place that shows code
 * outside the source column uses this one. */
export function Code(props: {
  path: string;
  text: string;
  class?: string | undefined;
  title?: string | undefined;
  mark?: MarkRange | undefined;
  markClass?: string | undefined;
}) {
  const tokens = createHighlight(
    () => props.path,
    () => props.text,
  );
  const lines = () => linesOf(props.text, tokens(), props.mark);
  return (
    <code class={props.class} title={props.title}>
      <For each={lines()}>
        {(parts, index) => (
          <>
            <For each={parts}>{(part) => <PartView part={part} markClass={props.markClass} />}</For>
            <Show when={index() < lines().length - 1}>{LINE_END}</Show>
          </>
        )}
      </For>
    </code>
  );
}
