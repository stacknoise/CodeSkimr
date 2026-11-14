import { runCommand } from "../../shared/api/commands";
import { CANCEL_COMMAND } from "./consts";

/** Asks the server to stop the running scan of a project at the next stage boundary. The stream
 * reports when it has stopped; a failed call changes nothing on the screen. */
export async function cancelScan(project: number): Promise<void> {
  try {
    await runCommand(CANCEL_COMMAND, { project });
  } catch {
    // Kept on purpose: the screen shows what the stream tells, not what this call answered.
  }
}
