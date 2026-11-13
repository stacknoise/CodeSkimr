import { CODE_PARAMETER } from "./consts";

let waiting: string | null = null;

/** Takes the launch code out of the address and removes the fragment from it. This is the first
 * thing the UI does, before any other script runs (ADR 0028): the code is then in no address
 * the user sees, copies or has in the history. */
export function captureLaunchCode(
  location: Pick<Location, "hash" | "pathname" | "search">,
  history: Pick<History, "replaceState">,
): string | null {
  const code = new URLSearchParams(location.hash.slice(1)).get(CODE_PARAMETER);
  if (location.hash !== "") {
    history.replaceState(null, document.title, location.pathname + location.search);
  }
  return code;
}

waiting = captureLaunchCode(window.location, window.history);

/** The launch code of the address, once. */
export function takeLaunchCode(): string | null {
  const code = waiting;
  waiting = null;
  return code;
}
