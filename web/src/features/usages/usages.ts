import { createSignal } from "solid-js";
import { navigate } from "../../app/router";
import { EXPLORER_ROUTE } from "../../app/consts";
import { runCommand } from "../../shared/api/commands";
import { focusSymbol, requestFocus, setFocusSymbol } from "../../shared/navigation/navigation";
import { COMMAND, TYPE_KINDS } from "../explorer/consts";
import { failed, type Failure } from "../explorer/failure";
import type { SymbolInfo } from "../explorer/types";
import {
  DEFAULT_DEPTH,
  DEFAULT_SHOW_TESTS,
  NO_HIT,
  USAGES_CODE_COMMAND,
  USAGES_COMMAND,
  WAY,
  type Way,
} from "./consts";
import { codeByPlace } from "./hits";
import type { CodeAnswer, Place, UsagesAnswer } from "./types";

const [way, setWay] = createSignal<Way>(WAY.both);
const [depth, setDepth] = createSignal(DEFAULT_DEPTH);
const [showTests, setShowTests] = createSignal(DEFAULT_SHOW_TESTS);
const [answer, setAnswer] = createSignal<UsagesAnswer | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [loading, setLoading] = createSignal(false);
const [selected, setSelected] = createSignal(NO_HIT);
const [code, setCode] = createSignal<Map<string, string>>(new Map());

/** Counts the questions asked; the answer to an earlier one is dropped. */
let generation = 0;

/** The symbol the screen shows, `null` until one is chosen. */
export { focusSymbol };
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
/** The code lines of the hits by place; empty until read, or if the code cannot be read. */
export const hitCode = code;

/** Reads the usages of `symbol` with the settings of the toolbar. */
export async function loadUsages(project: number, symbol: number): Promise<void> {
  generation += 1;
  const mine = generation;
  setLoading(true);
  setFailure(null);
  const input = {
    project,
    symbol,
    way: way(),
    depth: depth(),
    include_tests: showTests(),
  };
  try {
    const read = await runCommand<UsagesAnswer>(USAGES_COMMAND, input);
    if (mine === generation) {
      setSelected(NO_HIT);
      setCode(new Map());
      setAnswer(read);
      void loadCode(input, mine);
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

/** Reads the code lines of the hits after the list is shown. They are an addition: if the code
 * cannot be read (a Git project, a missing right) the list keeps the file and line. */
async function loadCode(input: Record<string, unknown>, mine: number): Promise<void> {
  try {
    const read = await runCommand<CodeAnswer>(USAGES_CODE_COMMAND, input);
    if (mine === generation) {
      setCode(codeByPlace(read.lines));
    }
  } catch {
    // The hit list stays as it is.
  }
}

/** Forgets the symbol and the answer, so a screen opened later does not show another project's
 * usages. */
export function clearUsages(): void {
  generation += 1;
  setFocusSymbol(null);
  setAnswer(null);
  setCode(new Map());
  setFailure(null);
  setLoading(false);
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
