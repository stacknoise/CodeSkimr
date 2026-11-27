import { Show } from "solid-js";
import styles from "./Parts.module.css";

/** One figure of a row of figures: a label, the number, a small text next to it and a hint under
 * it (mocks 29 and erw-auswirkung). */
export function Figure(props: { label: string; value: string; small?: string; hint?: string }) {
  return (
    <div class={styles.figure}>
      <span class={styles.figureLabel}>{props.label}</span>
      <span class={styles.figureValue}>
        {props.value}
        <Show when={props.small}>{(small) => <small>{small()}</small>}</Show>
      </span>
      <Show when={props.hint}>{(hint) => <span class={styles.figureHint}>{hint()}</span>}</Show>
    </div>
  );
}
