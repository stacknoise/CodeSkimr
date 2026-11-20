import { createSignal, onCleanup } from "solid-js";
import { NOW_TICK_MS } from "./consts";

/** The current time in milliseconds, renewed every `NOW_TICK_MS`, so that "3 min ago" keeps up
 * while the screen stays open. */
export function useNow(): () => number {
  const [now, setNow] = createSignal(Date.now());
  const timer = setInterval(() => setNow(Date.now()), NOW_TICK_MS);
  onCleanup(() => clearInterval(timer));
  return now;
}
