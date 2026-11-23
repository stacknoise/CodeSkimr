import { api } from "./client";
import { COMMANDS_PATH, METHOD_POST, TIMED_COMMANDS, TIMED_LEVELS } from "./consts";
import { recordTime } from "./timings";

/** The budget that the round trip of a command measures, if any: a command with levels measures
 * only the ones in `TIMED_LEVELS`. */
function timedBudget(id: string, input: unknown): string | undefined {
  const levels = TIMED_LEVELS[id];
  if (levels !== undefined) {
    const level = (input as { level?: string } | undefined)?.level;
    if (!levels.includes(level)) {
      return undefined;
    }
  }
  return TIMED_COMMANDS[id];
}

/** Runs a command of the registry on the local server (`POST /api/v1/commands/{id}`) and returns
 * its output. A `signal` lets the caller give up waiting. The round trip of a command that
 * measures a time budget is remembered for the performance screen and the status bar. Commands
 * that change something need the token, which `api` sends.
 *
 * @throws ApiError with the registered code of the failure.
 */
export async function runCommand<Output>(
  id: string,
  input?: unknown,
  signal?: AbortSignal,
): Promise<Output> {
  const started = performance.now();
  const output = await api<Output>(`${COMMANDS_PATH}${id}`, {
    method: METHOD_POST,
    body: input,
    ...(signal === undefined ? {} : { signal }),
  });
  const budget = timedBudget(id, input);
  if (budget !== undefined) {
    recordTime(budget, performance.now() - started);
  }
  return output;
}
