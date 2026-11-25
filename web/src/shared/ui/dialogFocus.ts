import { onCleanup, onMount } from "solid-js";
import { useKeydown } from "../keys/keydown";
import { FOCUSABLE, TAB_KEY } from "./consts";

/** Keeps Tab inside the dialog: from the last field it goes to the first, and back. */
function trapTab(event: KeyboardEvent, dialog: HTMLElement | undefined): void {
  if (event.key !== TAB_KEY || dialog === undefined) {
    return;
  }
  const fields = Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE));
  const first = fields[0];
  const last = fields[fields.length - 1];
  const edge = event.shiftKey ? first : last;
  if (first !== undefined && last !== undefined && document.activeElement === edge) {
    event.preventDefault();
    (event.shiftKey ? last : first).focus();
  }
}

/** The focus of a modal dialog: it starts on `initial` (if given), Tab stays inside, and when the
 * dialog closes the focus goes back to where it was when the dialog opened. */
export function useDialogFocus(
  dialog: () => HTMLElement | undefined,
  initial?: () => HTMLElement | undefined,
): void {
  const before = document.activeElement;
  useKeydown((event) => trapTab(event, dialog()));
  onMount(() => initial?.()?.focus());
  onCleanup(() => {
    if (before instanceof HTMLElement && before.isConnected) {
      before.focus();
    }
  });
}
