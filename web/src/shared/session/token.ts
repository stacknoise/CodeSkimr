import { TOKEN_STORAGE_KEY } from "./consts";

let memory: string | null = null;

function storage(): Storage | undefined {
  try {
    return window.sessionStorage;
  } catch {
    return undefined;
  }
}

/** The installation token of this tab, or `null`. It lives in `sessionStorage` only, never in a
 * URL, a cookie or `localStorage` (ADR 0028); where the browser does not allow that, in memory. */
export function getToken(): string | null {
  try {
    return storage()?.getItem(TOKEN_STORAGE_KEY) ?? memory;
  } catch {
    return memory;
  }
}

/** Keeps the token for this tab. */
export function storeToken(token: string): void {
  memory = token;
  try {
    storage()?.setItem(TOKEN_STORAGE_KEY, token);
  } catch {
    // The memory copy is enough until the page is reloaded.
  }
}

/** Forgets the token, for example after the server refused it. */
export function invalidateToken(): void {
  memory = null;
  try {
    storage()?.removeItem(TOKEN_STORAGE_KEY);
  } catch {
    // Nothing is left to remove.
  }
  onInvalidated?.();
}

let onInvalidated: (() => void) | undefined;

/** Registers what happens when the token is dropped; the session state listens here. */
export function whenInvalidated(listener: () => void): void {
  onInvalidated = listener;
}
