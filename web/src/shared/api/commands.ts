import { api } from "./client";
import { COMMANDS_PATH, METHOD_POST } from "./consts";

/** Runs a command of the registry on the local server (`POST /api/v1/commands/{id}`) and returns
 * its output. Commands that change something need the token, which `api` sends.
 *
 * @throws ApiError with the registered code of the failure.
 */
export function runCommand<Output>(id: string, input?: unknown): Promise<Output> {
  return api<Output>(`${COMMANDS_PATH}${id}`, { method: METHOD_POST, body: input });
}
