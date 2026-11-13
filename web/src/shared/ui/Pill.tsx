import type { JSX } from "solid-js";
import { TONE } from "./consts";
import { cx } from "./cx";
import styles from "./Pill.module.css";

/** Meaning of a pill, as in the mocks: `accent` marks a selection, the others a state. */
export type PillTone = (typeof TONE)[keyof typeof TONE];

/** A small label such as "Selected" or "current". `compact` is for miniatures. */
export function Pill(props: { tone?: PillTone; compact?: boolean; children: JSX.Element }) {
  return (
    <span
      class={cx(styles.pill, styles[props.tone ?? TONE.neutral], props.compact && styles.compact)}
    >
      {props.children}
    </span>
  );
}
