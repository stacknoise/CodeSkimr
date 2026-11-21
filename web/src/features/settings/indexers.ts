import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { INDEXERS_COMMAND } from "../projects/consts";
import type { ToolsInfo } from "../projects/types";

const [listing, setListing] = createSignal<ToolsInfo | null>(null);
const [checking, setChecking] = createSignal(false);
const [failed, setFailed] = createSignal(false);

/** The indexers of this computer as the last check found them; `null` before the first answer. */
export const indexers = listing;
/** Whether a check is running. */
export const indexersChecking = checking;
/** Whether the last check got no answer (the server could not be reached). */
export const indexersFailed = failed;

/** Checks the indexers of this computer again. A failure keeps what the screen shows. */
export async function checkIndexers(): Promise<void> {
  if (checking()) {
    return;
  }
  setChecking(true);
  try {
    setListing(await runCommand<ToolsInfo>(INDEXERS_COMMAND));
    setFailed(false);
  } catch {
    setFailed(true);
  } finally {
    setChecking(false);
  }
}

/** The command in an install text: the part between the first pair of quotes. */
export function commandOf(install: string): string | null {
  return /"([^"]+)"/.exec(install)?.[1] ?? null;
}
