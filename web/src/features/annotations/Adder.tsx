import { Show, type JSX } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import styles from "./Tabs.module.css";

/** The foot of the tab Notes and the tab Tasks: the button that adds one (mock 14: "+ Add note"),
 * or the form once it was pressed, and the hint at the keys. */
export function Adder(props: {
  adding: boolean;
  label: string;
  onAdd: () => void;
  children: JSX.Element;
}) {
  return (
    <>
      <Show
        when={props.adding}
        fallback={
          <button type="button" class={cx(styles.primary, styles.plus)} onClick={props.onAdd}>
            {props.label}
          </button>
        }
      >
        {props.children}
      </Show>
      <p class={styles.keys}>{t(I18N_KEYS.ANNOTATIONS_KEYS_HINT)}</p>
    </>
  );
}
