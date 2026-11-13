import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { COMMAND } from "./consts";
import { failed, type Failure } from "./failure";
import type { SourceInfo } from "./types";

const [info, setInfo] = createSignal<SourceInfo | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [loading, setLoading] = createSignal(false);

/** Counts the reads; the answer of an earlier one is dropped. */
let generation = 0;

/** The source of the shown symbol, `null` while it is read or nothing is shown. */
export const sourceInfo = info;
/** Why the source could not be read. */
export const sourceError = failure;
/** Whether the source is being read. */
export const sourceLoading = loading;

/** Forgets the source: nothing is selected any more. */
export function clearSource(): void {
  generation += 1;
  setInfo(null);
  setFailure(null);
  setLoading(false);
}

/** Reads the lines of the declaration of `symbol`; the old ones stay out of the way until the
 * new ones are there, so the column never shows the code of another symbol. */
export async function loadSource(project: number, symbol: number): Promise<void> {
  generation += 1;
  const mine = generation;
  setInfo(null);
  setFailure(null);
  setLoading(true);
  try {
    const read = await runCommand<SourceInfo>(COMMAND.source, { project, symbol });
    if (mine === generation) {
      setInfo(read);
    }
  } catch (error) {
    if (mine === generation) {
      setFailure(failed(error));
    }
  } finally {
    if (mine === generation) {
      setLoading(false);
    }
  }
}
