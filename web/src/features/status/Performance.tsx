import { createSignal, Show } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { useLayer } from "../../shared/keys/layers";
import { Budgets } from "../scan/Budgets";
import { CANCEL_KEY } from "../scan/consts";
import { performance } from "../scan/performance";
import styles from "./Performance.module.css";

/** The entry "Performance" of the status bar: it says how many targets were missed, and a click
 * opens the list of the targets with their measured values (mock 23, ADR 0096). The targets are
 * those of the whole app, not of a project. */
export function Performance() {
  const [open, setOpen] = createSignal(false);
  const state = () => performance();
  useLayer(open);
  useKeydown((event) => {
    if (open() && event.key === CANCEL_KEY && !isTyping(event.target)) {
      setOpen(false);
    }
  });
  return (
    <>
      <button
        type="button"
        class={styles.entry}
        aria-expanded={open()}
        aria-haspopup="dialog"
        onClick={() => setOpen(!open())}
      >
        {state().missed > 0
          ? t(I18N_KEYS.STATUS_PERFORMANCE_MISSED, { count: state().missed })
          : t(I18N_KEYS.STATUS_PERFORMANCE)}
      </button>
      <Show when={open()}>
        <div class={styles.popup} role="dialog" aria-label={t(I18N_KEYS.SCAN_BUDGETS_TITLE)}>
          <Budgets
            targets={state().targets}
            measured={state().measured}
            unrated={state().unrated}
          />
        </div>
      </Show>
    </>
  );
}
