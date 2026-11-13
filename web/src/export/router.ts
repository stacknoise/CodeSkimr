import { createSignal } from "solid-js";
import { POPSTATE_EVENT } from "../app/consts";
import { HASH_PREFIX, START_PATH } from "./consts";

/** The path the address names: `#/graph` is `/graph`. A page without a fragment shows the start
 * screen. */
function readPath(): string {
  const path = window.location.hash.slice(HASH_PREFIX.length);
  return path === "" ? START_PATH : path;
}

const [path, setPath] = createSignal(readPath());

window.addEventListener(POPSTATE_EVENT, () => setPath(readPath()));

/** The path of the page, for choosing the screen. */
export const currentPath = path;

/** Shows the screen of `to`. The export is one file, so the path goes into the fragment: the
 * address of a file cannot take another path. */
export function navigate(to: string): void {
  if (to !== path()) {
    window.history.pushState(null, document.title, `${HASH_PREFIX}${to}`);
    setPath(to);
  }
}

/** The address a link to `path` carries: the fragment, as the file cannot take another path. */
export function hrefOf(path: string): string {
  return `${HASH_PREFIX}${path}`;
}
