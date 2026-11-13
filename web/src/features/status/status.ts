import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { STATUS_COMMAND } from "./consts";
import type { StatusInfo } from "./types";

const [info, setInfo] = createSignal<StatusInfo | null>(null);

/** The last measurements of the program, `null` until the first answer. */
export const statusInfo = info;

/** Reads the measurements again. A failure keeps the old values: the connection text of the
 * status bar already says when the server cannot be reached. */
export async function refreshStatus(): Promise<void> {
  try {
    setInfo(await runCommand<StatusInfo>(STATUS_COMMAND));
  } catch {
    // Kept on purpose: see above.
  }
}
