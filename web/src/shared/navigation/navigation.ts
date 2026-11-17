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
const [usagesOrigin, setUsagesOrigin] = createSignal<number | null>(null);

/** The place the explorer has to show next, `null` if none is asked for. */
export const pendingFocus = pending;
/** Asks the explorer to show a place; the explorer takes it and clears it. */
export const requestFocus = setPending;
/** The symbol the usages screen shows, `null` until one is chosen. */
export const focusSymbol = usagesSymbol;
/** Sets or forgets the symbol of the usages screen without leaving the current screen. */
export const setFocusSymbol = setUsagesSymbol;

/** The symbol the user left the explorer from to reach the usages screen; the way back leads to
 * it, however far the user moved through the graph meanwhile. `null` until one is chosen. */
export const originSymbol = usagesOrigin;
/** Forgets the symbol the way back leads to. */
export const clearOrigin = () => setUsagesOrigin(null);

/** Shows the usages of a symbol from the explorer; the symbol is where the way back leads to. */
export function showUsages(symbol: number): void {
  setUsagesOrigin(symbol);
  setUsagesSymbol(symbol);
  navigate(USAGES_ROUTE.path);
}

/** Moves the usages screen to another symbol, from a card of the graph; the way back stays. */
export function moveUsages(symbol: number): void {
  setUsagesSymbol(symbol);
}
