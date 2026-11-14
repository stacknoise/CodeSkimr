import { api } from "./client";
import { COMMANDS_PATH, METHOD_POST, TIMED_COMMANDS } from "./consts";
import { recordTime } from "./timings";

/** Runs a command of the registry on the local server (`POST /api/v1/commands/{id}`) and returns
 * its output. The round trip of a command that measures a time budget is remembered for the
 * performance screen and the status bar. Commands that change something need the token, which `api` sends.
 *
 * @throws ApiError with the registered code of the failure.
 */
export async function runCommand<Output>(id: string, input?: unknown): Promise<Output> {
  const started = performance.now();
  const output = await api<Output>(`${COMMANDS_PATH}${id}`, { method: METHOD_POST, body: input });
  const budget = TIMED_COMMANDS[id];
  if (budget !== undefined) {
    recordTime(budget, performance.now() - started);
  }
  return output;
}
