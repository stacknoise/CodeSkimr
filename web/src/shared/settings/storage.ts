/** Reads a setting from `localStorage`; `null` if there is none or the storage is not usable. */
export function readSetting(
  key: string,
  storage: Storage | undefined = safeLocal(),
): string | null {
  try {
    return storage?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

/** Writes a setting, or removes it for `null`. A storage that cannot be used is ignored: the
 * setting then lasts until the page is closed. */
export function writeSetting(
  key: string,
  value: string | null,
  storage: Storage | undefined = safeLocal(),
): void {
  try {
    if (value === null) {
      storage?.removeItem(key);
    } else {
      storage?.setItem(key, value);
    }
  } catch {
    // Private windows and blocked site data: nothing to do.
  }
}

/** The `localStorage` of the page, or `undefined` if the browser does not let the page use it. */
export function safeLocal(): Storage | undefined {
  try {
    return window.localStorage;
  } catch {
    return undefined;
  }
}
