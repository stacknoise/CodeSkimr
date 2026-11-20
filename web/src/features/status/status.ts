import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { noteReachable } from "../../shared/session/session";
import { STATUS_COMMAND } from "./consts";
import type { StatusInfo } from "./types";

const [info, setInfo] = createSignal<StatusInfo | null>(null);

/** The last measurements of the program, `null` until the first answer. */
export const statusInfo = info;

/** Reads the measurements again. A failure keeps the old values. A call that gets no answer
 * at all tells the session that the server is gone, any answer that the server gave, even a
 * refusal, tells that it is back. */
export async function refreshStatus(): Promise<void> {
  try {
    setInfo(await runCommand<StatusInfo>(STATUS_COMMAND));
    noteReachable(true);
  } catch (error) {
    noteReachable(error instanceof ApiError);
  }
}
