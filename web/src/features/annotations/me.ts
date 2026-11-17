import { createSignal } from "solid-js";
import { ME_KEY } from "./consts";

function stored(): string | null {
  try {
    return window.localStorage.getItem(ME_KEY);
  } catch {
    return null;
  }
}

const [name, setName] = createSignal<string | null>(stored());

/** The name the server gave as author of the last annotation made in this browser; the server has
 * no command that says who the caller is, so "Mine" on the board is read from it. `null` until
 * the user made one. */
export const myName = name;

/** Remembers the author of an annotation that this browser just made. */
export function rememberAuthor(author: string): void {
  setName(author);
  try {
    window.localStorage.setItem(ME_KEY, author);
  } catch {
    // Without storage the name is kept until the page is closed.
  }
}

/** Forgets the name; for tests. */
export function forgetMe(): void {
  setName(null);
  try {
    window.localStorage.removeItem(ME_KEY);
  } catch {
    // Nothing was stored.
  }
}
