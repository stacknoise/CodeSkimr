import { createSignal } from "solid-js";
import { POPSTATE_EVENT } from "./consts";

const [path, setPath] = createSignal(window.location.pathname);

window.addEventListener(POPSTATE_EVENT, () => setPath(window.location.pathname));

/** The path of the page, for choosing the screen. */
export const currentPath = path;

/** Shows the screen of `to` without loading the page again. */
export function navigate(to: string): void {
  if (to !== path()) {
    window.history.pushState(null, document.title, to);
    setPath(to);
  }
}
