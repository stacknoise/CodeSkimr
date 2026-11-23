import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { t } from "../i18n/i18n";
import { I18N_KEYS } from "../i18n/keys";
import { NoticeBox } from "./NoticeBox";
import { SLOW_LOAD_DELAY_MS } from "./consts";

/** Whether `waiting` has been true for longer than the delay, so a quick answer shows no flicker. */
export function useSlow(waiting: () => boolean): () => boolean {
  const [slow, setSlow] = createSignal(false);
  createEffect(() => {
    if (!waiting()) {
      setSlow(false);
      return;
    }
    const timer = setTimeout(() => setSlow(true), SLOW_LOAD_DELAY_MS);
    onCleanup(() => clearTimeout(timer));
  });
  return slow;
}

/** Says that a screen is still loading, once the answer takes longer than the delay. */
export function SlowLoad(props: { waiting: () => boolean; class?: string | undefined }) {
  const slow = useSlow(props.waiting);
  return (
    <div role="status">
      <Show when={slow()}>
        <NoticeBox class={props.class} text={t(I18N_KEYS.LOADING_SLOW)} />
      </Show>
    </div>
  );
}
