import { createEffect, createRoot, on } from "solid-js";
import { runCommand } from "../api/commands";
import { isLanguage, currentLanguage, setLanguage, systemLanguage } from "../i18n/i18n";
import { KIND } from "../session/consts";
import { sessionState } from "../session/session";
import { chosenTheme, followSystem, isTheme, selectTheme } from "../theme/theme";
import { PREFERENCES_GET, PREFERENCES_SET } from "./consts";

/** The choices of the user, as the server keeps them. `null` is no choice. */
export interface Preferences {
  theme: string | null;
  language: string | null;
}

/** What the browser shows now. A language that is the system language counts as no choice, since
 * the UI writes it to `localStorage` at start whether or not the user ever chose it. */
function local(): Preferences {
  const language = currentLanguage();
  return { theme: chosenTheme(), language: language === systemLanguage() ? null : language };
}

let known: Preferences = { theme: null, language: null };

function same(a: Preferences, b: Preferences): boolean {
  return a.theme === b.theme && a.language === b.language;
}

/** Takes over what the server holds: a theme or language that this UI offers is applied, anything
 * else is ignored. */
function adopt(server: Preferences): void {
  if (isTheme(server.theme)) {
    selectTheme(server.theme);
  } else if (server.theme === null) {
    followSystem();
  }
  if (server.language !== null && isLanguage(server.language)) {
    setLanguage(server.language);
  }
}

/** Sends the choices to the server. A server that does not answer keeps the old ones; the browser
 * keeps working with its own copy. */
async function push(next: Preferences): Promise<void> {
  if (same(next, known)) {
    return;
  }
  try {
    known = await runCommand<Preferences>(PREFERENCES_SET, next);
  } catch {
    // The copy in the browser still holds the choice; the next change tries again.
  }
}

/** Reads the choices from the server: if it holds any they win over the copy of this browser, so
 * that every browser of the user looks the same. If it holds none, the choices of this browser
 * (made before the server kept them) go there. */
async function pull(): Promise<void> {
  let server: Preferences;
  try {
    server = await runCommand<Preferences>(PREFERENCES_GET);
  } catch {
    return;
  }
  known = server;
  if (server.theme !== null || server.language !== null) {
    adopt(server);
  } else {
    await push(local());
  }
  createRoot(() => {
    createEffect(on([chosenTheme, currentLanguage], () => void push(local()), { defer: true }));
  });
}

let started = false;

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

/** Test hook: forgets what the module knows. */
export function resetPreferencesForTest(): void {
  started = false;
  known = { theme: null, language: null };
}
