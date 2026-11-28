import { createEffect, createSignal, onCleanup } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { NETWORK_PATH, SCOPE_COMMAND, SUGGEST_DELAY_MS } from "./consts";
import type { AbapGitInfo } from "./scope";

/** What the folder typed in the window "New project" is, as far as `scope.inspect` can tell. */
interface Inspected {
  abapgit: AbapGitInfo | null;
}

/** Looks into the typed folder after a pause in typing; `null` while nothing is known, for a Git
 * address and for a network path (which would make the server reach out), and when the server does
 * not answer: the hint is a help, never a block (ADR 0113). */
export function useInspected(source: () => string, folder: () => boolean) {
  const [found, setFound] = createSignal<AbapGitInfo | null>(null);
  let asked = 0;
  createEffect(() => {
    const text = source().trim();
    const ticket = ++asked;
    const wanted = folder() && text !== "" && !NETWORK_PATH.test(text);
    if (!wanted) {
      setFound(null);
      return;
    }
    const timer = setTimeout(() => {
      void runCommand<Inspected>(SCOPE_COMMAND.inspect, { source: text })
        .then((answer) => answer.abapgit)
        .catch(() => null)
        .then((abapgit) => {
          if (ticket === asked) {
            setFound(abapgit);
          }
        });
    }, SUGGEST_DELAY_MS);
    onCleanup(() => clearTimeout(timer));
  });
  return found;
}
