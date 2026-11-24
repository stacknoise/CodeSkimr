import { type Accessor, createSignal, onCleanup } from "solid-js";
import { COPIED_MS, COPY } from "./consts";

/** What a copy to the clipboard came to: it worked, it failed, or nothing happened lately. */
export type CopyState = (typeof COPY)[keyof typeof COPY] | null;

/** Puts text on the clipboard and keeps the state of the last try for a moment, so a button can say
 * "Copied" or "Not copied"; `onFailed` runs on a failure. A browser without a clipboard counts as a failure. */
export function createCopier(onFailed?: () => void): {
  state: Accessor<CopyState>;
  copy: (text: string) => void;
} {
  const [state, setState] = createSignal<CopyState>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(timer));
  const show = (next: CopyState) => {
    setState(next);
    if (next === COPY.failed) onFailed?.();
    clearTimeout(timer);
    timer = setTimeout(() => setState(null), COPIED_MS);
  };
  const copy = (text: string) => {
    if (navigator.clipboard === undefined) {
      show(COPY.failed);
      return;
    }
    navigator.clipboard
      .writeText(text)
      .then(() => show(COPY.copied))
      .catch(() => show(COPY.failed));
  };
  return { state, copy };
}
