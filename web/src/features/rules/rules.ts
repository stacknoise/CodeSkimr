import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { failed, type Failure } from "../explorer/failure";
import { COMMAND } from "./consts";
import type { RulesOut } from "./types";

const [data, setData] = createSignal<RulesOut | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
/** Counts the loads, so that an answer for a project that is no longer shown is dropped. */
let latest = 0;

/** What the server said about the rules of the project. */
export const rulesData = data;
/** The failure of the last read; `null` if it worked. */
export const rulesFailure = failure;

/** Reads the rules of a project and their violations. */
export async function loadRules(project: number): Promise<void> {
  const mine = ++latest;
  setData(null);
  setFailure(null);
  try {
    const read = await runCommand<RulesOut>(COMMAND.show, { project });
    if (mine === latest) {
      setData(read);
    }
  } catch (error) {
    if (mine === latest) {
      setFailure(failed(error));
    }
  }
}

/** Forgets what was read, when the screen is left. */
export function resetRules(): void {
  latest += 1;
  setData(null);
  setFailure(null);
}
