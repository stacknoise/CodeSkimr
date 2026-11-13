import { onCleanup, onMount } from "solid-js";

/** Elements in which typing must not trigger a shortcut. */
const TYPING_TAGS: readonly string[] = ["INPUT", "TEXTAREA", "SELECT"];

/** Whether the target of a key event is a field the user types into. */
export function isTyping(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && TYPING_TAGS.includes(target.tagName);
}

/** Listens to the keys of the whole document while the calling screen is shown. */
export function useKeydown(listener: (event: KeyboardEvent) => void): void {
  onMount(() => document.addEventListener("keydown", listener));
  onCleanup(() => document.removeEventListener("keydown", listener));
}
