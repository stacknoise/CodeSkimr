import { createSignal } from "solid-js";
import { USAGES_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";

/** Where another screen (the quick search, the usages) wants the explorer tree to go: a type, or a
 * member of it. */
export interface Focus {
  type: number;
  member: number | null;
  /** Kind of the member, to open the group that holds it. */
  kind: string | null;
}

const [pending, setPending] = createSignal<Focus | null>(null);
const [usagesSymbol, setUsagesSymbol] = createSignal<number | null>(null);

/** The place the explorer has to show next, `null` if none is asked for. */
export const pendingFocus = pending;
/** Asks the explorer to show a place; the explorer takes it and clears it. */
export const requestFocus = setPending;
/** The symbol the usages screen shows, `null` until one is chosen. */
export const focusSymbol = usagesSymbol;
/** Sets or forgets the symbol of the usages screen without leaving the current screen. */
export const setFocusSymbol = setUsagesSymbol;

/** Shows the usages of a symbol: from the explorer, or from a card of the graph. */
export function showUsages(symbol: number): void {
  setUsagesSymbol(symbol);
  navigate(USAGES_ROUTE.path);
}
