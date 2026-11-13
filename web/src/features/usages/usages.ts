import { createSignal } from "solid-js";
import { navigate } from "../../app/router";
import { EXPLORER_ROUTE, USAGES_ROUTE } from "../../app/consts";
import { runCommand } from "../../shared/api/commands";
import { COMMAND, TYPE_KINDS } from "../explorer/consts";
import { requestFocus } from "../explorer/explorer";
import { failed, type Failure } from "../explorer/failure";
import type { SymbolInfo } from "../explorer/types";
import { DEFAULT_DEPTH, DEFAULT_SHOW_TESTS, NO_HIT, USAGES_COMMAND, WAY, type Way } from "./consts";
import type { Place, UsagesAnswer } from "./types";

const [focus, setFocus] = createSignal<number | null>(null);
const [way, setWay] = createSignal<Way>(WAY.both);
const [depth, setDepth] = createSignal(DEFAULT_DEPTH);
const [showTests, setShowTests] = createSignal(DEFAULT_SHOW_TESTS);
const [answer, setAnswer] = createSignal<UsagesAnswer | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [loading, setLoading] = createSignal(false);
const [selected, setSelected] = createSignal(NO_HIT);

/** Counts the questions asked; the answer to an earlier one is dropped. */
let generation = 0;

/** The symbol the screen shows, `null` until one is chosen. */
export const focusSymbol = focus;
/** The directions, the depth and the tests flag of the toolbar. */
export const direction = way;
export const setDirection = setWay;
export const walkDepth = depth;
export const setWalkDepth = setDepth;
export const testsShown = showTests;
export const setTestsShown = setShowTests;
/** The last answer, `null` until it has been read. */
export const usagesAnswer = answer;
/** Why the last read failed. */
export const usagesFailure = failure;
export const usagesLoading = loading;
/** Index of the selected hit of the list, `NO_HIT` if none is. */
export const selectedHit = selected;
export const setSelectedHit = setSelected;

/** Reads the usages of `symbol` with the settings of the toolbar. */
export async function loadUsages(project: number, symbol: number): Promise<void> {
  generation += 1;
  const mine = generation;
  setLoading(true);
  setFailure(null);
  try {
    const read = await runCommand<UsagesAnswer>(USAGES_COMMAND, {
      project,
      symbol,
      way: way(),
      depth: depth(),
      include_tests: showTests(),
    });
    if (mine === generation) {
      setSelected(NO_HIT);
      setAnswer(read);
    }
  } catch (error) {
    if (mine === generation) {
      setAnswer(null);
      setFailure(failed(error));
    }
  } finally {
    if (mine === generation) {
      setLoading(false);
    }
  }
}

/** Forgets the symbol and the answer, so a screen opened later does not show another project's
 * usages. */
export function clearUsages(): void {
  generation += 1;
  setFocus(null);
  setAnswer(null);
  setFailure(null);
  setLoading(false);
}

/** Shows the usages of a symbol: from the explorer, or from a card of the graph. */
export function showUsages(symbol: number): void {
  setFocus(symbol);
  navigate(USAGES_ROUTE.path);
}

/** Shows the symbol in the explorer: a type itself, a member within the type that holds it. The
 * type of a member comes from the breadcrumb of the symbol. */
export async function openInExplorer(project: number, place: Place): Promise<void> {
  if (TYPE_KINDS.includes(place.kind)) {
    requestFocus({ type: place.id, member: null, kind: null });
  } else {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol: place.id });
    const owner = info.breadcrumb[info.breadcrumb.length - 1]?.id ?? place.id;
    requestFocus({ type: owner, member: owner === place.id ? null : place.id, kind: place.kind });
  }
  navigate(EXPLORER_ROUTE.path);
}
