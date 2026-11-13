import { KIND_LETTER, KIND_LETTER_OTHER, KIND_TONE, KIND_TONE_OTHER } from "./consts";
import styles from "./KindBadge.module.css";
import { cx } from "../../shared/ui/cx";

/** The small coloured letter in front of a symbol (mock 2). The tone says the kind; the letter
 * repeats it, so colour is not the only carrier. */
export function KindBadge(props: { kind: string }) {
  const tone = () => KIND_TONE[props.kind] ?? KIND_TONE_OTHER;
  return (
    <span class={cx(styles.badge, styles[tone()])} title={props.kind}>
      {KIND_LETTER[props.kind] ?? KIND_LETTER_OTHER}
    </span>
  );
}
