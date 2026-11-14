import { createEffect, createRoot, on } from "solid-js";
import { runCommand } from "../api/commands";
import { currentLanguage, isLanguage, setLanguage, systemLanguage } from "../i18n/i18n";
import { KIND } from "../session/consts";
import { sessionState } from "../session/session";
import { chosenTheme, isTheme, selectTheme } from "../theme/theme";
import { PREFERENCES_GET, PREFERENCES_SET } from "./consts";

/** The choices of the user, as the server keeps them. `null` is no choice. */
export interface Preferences {
  theme: string | null;
  language: string | null;
}

/** What the server holds, once it was read or answered. */
let known: Preferences | null = null;
/** The changes of this browser that the server does not hold yet. */
let pending: Partial<Preferences> = {};
let sending = false;
let started = false;

function same(a: Preferences, b: Preferences): boolean {
  return a.theme === b.theme && a.language === b.language;
}

/** Reads the choices from the server. `false` if it does not answer. */
async function load(): Promise<boolean> {
  try {
    known = await runCommand<Preferences>(PREFERENCES_GET);
    return true;
  } catch {
    return false;
  }
}

/** Sends `wanted` until the server holds it. Changes that come in while a call is under way are
 * picked up by the loop, so the server ends with the last choice and the answers cannot cross.
 * A server that does not answer keeps the old choices; the browser works with its own copy and
 * the next change tries again, reading the server first if that was not possible before. */
async function flush(): Promise<void> {
  if (sending) {
    return;
  }
  sending = true;
  try {
    if (known === null && !(await load())) {
      return;
    }
    while (known !== null && !same({ ...known, ...pending }, known)) {
      known = await runCommand<Preferences>(PREFERENCES_SET, { ...known, ...pending });
    }
    pending = {};
  } catch {
    // The copy in the browser still holds the choice.
  } finally {
    sending = false;
  }
}

/** Records a change of this browser and sends it. Only the changed value is replaced, so a choice
 * that another browser stored (a language, say) is not lost. */
function change(patch: Partial<Preferences>): void {
  pending = { ...pending, ...patch };
  void flush();
}

/** Takes over what the server holds, value by value: a theme or language that this UI offers is
 * applied, anything else is ignored. Returns what this browser holds that the server does not,
 * so that it can be sent. */
function adopt(server: Preferences): Partial<Preferences> {
  const extra: Partial<Preferences> = {};
  if (isTheme(server.theme)) {
    selectTheme(server.theme);
  } else if (server.theme === null && chosenTheme() !== null) {
    extra.theme = chosenTheme();
  }
  if (server.language !== null && isLanguage(server.language)) {
    setLanguage(server.language);
  } else if (server.language === null && currentLanguage() !== systemLanguage()) {
    // The UI writes the system language to `localStorage` at start whether or not the user
    // ever chose it, so only a language that differs from it counts as a choice.
    extra.language = currentLanguage();
  }
  return extra;
}

/** Reads the choices from the server: what it holds wins over the copy of this browser, so that
 * every browser of the user looks the same; what it lacks and this browser has goes there. */
async function pull(): Promise<void> {
  const answered = await load();
  const extra = known === null ? {} : adopt(known);
  createRoot(() => {
    createEffect(on(chosenTheme, (theme) => change({ theme }), { defer: true }));
    createEffect(on(currentLanguage, (language) => change({ language }), { defer: true }));
  });
  if (answered && Object.keys(extra).length > 0) {
    change(extra);
  }
}

/** Keeps theme and language on the server: once the session is connected the stored choices are
 * read, and every later choice is sent. Called once at start. */
export function syncPreferences(): void {
  if (started) {
    return;
  }
  started = true;
  createRoot(() => {
    let pulled = false;
    createEffect(() => {
      if (sessionState().kind === KIND.connected && !pulled) {
        pulled = true;
        void pull();
      }
    });
  });
}
