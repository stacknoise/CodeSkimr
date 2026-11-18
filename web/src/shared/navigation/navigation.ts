import { createSignal } from "solid-js";
import { USAGES_ROUTE } from "../../app/consts";
import { navigate } from "../../app/router";

/** A type, or a member of it. */
export interface TypeFocus {
  type: number;
  member: number | null;
  /** Kind of the member, to open the group that holds it. */
  kind: string | null;
}

/** A row of the tree above the types: a namespace (by its name, or by the start of it when the map
 * joins several), or a layer. */
export type RowFocus = { namespace: string } | { layer: string };

/** Where another screen (the quick search, the usages, the maps) wants the explorer tree to go. */
export type Focus = TypeFocus | RowFocus;

const [pending, setPending] = createSignal<Focus | null>(null);
const [usagesSymbol, setUsagesSymbol] = createSignal<number | null>(null);
const [usagesOrigin, setUsagesOrigin] = createSignal<number | null>(null);
const [usagesBack, setUsagesBack] = createSignal<TypeFocus | null>(null);

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
/** The place of the explorer the user left, as it was (the table of the members of a type, or one
 * member); the way back leads exactly there. `null` if the explorer did not say. */
export const originPlace = usagesBack;
/** Forgets the symbol and the place the way back leads to. */
export function clearOrigin(): void {
  setUsagesOrigin(null);
  setUsagesBack(null);
}

/** Shows the usages of a symbol from the explorer; the symbol is where the way back leads to,
 * unless `back` names the exact place the explorer was at. */
export function showUsages(symbol: number, back: TypeFocus | null = null): void {
  setUsagesOrigin(symbol);
  setUsagesBack(back);
  setUsagesSymbol(symbol);
  navigate(USAGES_ROUTE.path);
}

/** Moves the usages screen to another symbol, from a card of the graph; the way back stays. */
export function moveUsages(symbol: number): void {
  setUsagesSymbol(symbol);
}
