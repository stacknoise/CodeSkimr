import { For } from "solid-js";
import { BREAK_AFTER } from "./consts";

/** A name that may break after a separator (`.`, `_`, `:`) instead of in the middle of a word. */
export function Breakable(props: { text: string }) {
  return (
    <For each={props.text.split(BREAK_AFTER)}>
      {(part) => (
        <>
          {part}
          <wbr />
        </>
      )}
    </For>
  );
}
