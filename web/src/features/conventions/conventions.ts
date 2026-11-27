import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { failed, type Failure } from "../explorer/failure";
import { COMMAND } from "./consts";
import type { ConventionsOut } from "./types";

const [answer, setAnswer] = createSignal<ConventionsOut | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [running, setRunning] = createSignal(false);
const [checkedAt, setCheckedAt] = createSignal<Date | null>(null);
/** Counts the runs, so that an answer for a project that is no longer shown is dropped. */
let latest = 0;

/** What the last check found; `null` until one ran for the project that is shown. */
export const conventionsAnswer = answer;
/** The failure of the last check; `null` if it worked. */
export const conventionsFailure = failure;
/** Whether a check is running. */
export const conventionsRunning = running;
/** When the last check was done. */
export const conventionsCheckedAt = checkedAt;

/** Checks the files of a project folder against the code rules. */
export async function runConventionsCheck(project: number): Promise<void> {
  const mine = ++latest;
  setFailure(null);
  setRunning(true);
  try {
    const read = await runCommand<ConventionsOut>(COMMAND.check, { project });
    if (mine === latest) {
      setAnswer(read);
      setCheckedAt(new Date());
    }
  } catch (error) {
    if (mine === latest) {
      setAnswer(null);
      setFailure(failed(error));
    }
  } finally {
    if (mine === latest) {
      setRunning(false);
    }
  }
}

/** Forgets what was found, when the project changes or the tab is left. */
export function resetConventions(): void {
  latest += 1;
  setAnswer(null);
  setFailure(null);
  setRunning(false);
  setCheckedAt(null);
}
