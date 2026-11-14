import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { COMMAND } from "../explorer/consts";
import type { SymbolInfo } from "../explorer/types";

const [info, setInfo] = createSignal<SymbolInfo | null>(null);
/** Counts the loads, so that the answer for a hit that is no longer chosen is dropped. */
let latest = 0;

/** The symbol the preview shows, with its documentation and metrics; `null` while it loads. */
export const previewInfo = info;

/** Loads the details of the chosen hit. A failure leaves the preview without them. */
export async function loadPreview(project: number, symbol: number): Promise<void> {
  const mine = ++latest;
  setInfo((current) => (current?.id === symbol ? current : null));
  try {
    const answer = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol });
    if (mine === latest) {
      setInfo(answer);
    }
  } catch {
    if (mine === latest) {
      setInfo(null);
    }
  }
}

/** Forgets the details; for tests and when the search closes. */
export function resetPreview(): void {
  latest += 1;
  setInfo(null);
}
