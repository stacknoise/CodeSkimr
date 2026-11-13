import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { COMMAND, GROUP } from "./consts";
import { groupKey, groupOfKind, memberKey, revealKeys, typeKey } from "./tree";
import type { Outline, SymbolInfo } from "./types";

/** Why a read failed: a registered code, or `null` if the server could not be reached. */
export type Failure = { code: ErrorCode | null };

const [outline, setOutline] = createSignal<Outline | null>(null);
const [outlineFailure, setOutlineFailure] = createSignal<Failure | null>(null);
const [symbolFailure, setSymbolFailure] = createSignal<Failure | null>(null);
const [loading, setLoading] = createSignal(false);
const [details, setDetails] = createSignal<ReadonlyMap<number, SymbolInfo>>(new Map());
const [expanded, setExpanded] = createSignal<ReadonlySet<string>>(new Set<string>());
const [selectedType, setSelectedType] = createSignal<number | null>(null);
const [selectedMember, setSelectedMember] = createSignal<number | null>(null);
const [filter, setFilter] = createSignal("");
const [cursor, setCursor] = createSignal<string | null>(null);
const [pending, setPending] = createSignal<Focus | null>(null);

/** Where another screen (the quick search) wants the tree to go: a type, or a member of it. */
export interface Focus {
  type: number;
  member: number | null;
  /** Kind of the member, to open the group that holds it. */
  kind: string | null;
}

/** Counts the projects read so far; an answer for an earlier one is dropped. */
let generation = 0;

/** The tree of the project, `null` until it has been read. */
export const treeOutline = outline;
/** The failure of reading the tree. It leaves nothing to show but the message. */
export const outlineError = outlineFailure;
/** The failure of reading one symbol. The tree stays; the details show the message. */
export const symbolError = symbolFailure;
/** Whether the tree is being read. */
export const outlineLoading = loading;
/** The symbols read so far, by id. */
export const symbolDetails = details;
/** Keys of the opened rows. */
export const expandedRows = expanded;
/** The type that the middle column shows. */
export const currentType = selectedType;
/** The member that is highlighted in the tree and in the table. */
export const currentMember = selectedMember;
/** The row of the tree that the keys act on. */
export const treeCursor = cursor;
export const setTreeCursor = setCursor;
/** The navigation that waits for the tree to be read. */
export const pendingFocus = pending;
export const requestFocus = setPending;
/** What the filter field holds. */
export const treeFilter = filter;
export const setTreeFilter = setFilter;

function failed(error: unknown): Failure {
  return { code: error instanceof ApiError ? error.code : null };
}

/** Reads the tree of `project` and forgets everything that was read for another one. */
export async function loadOutline(project: number): Promise<void> {
  generation += 1;
  const mine = generation;
  setOutline(null);
  setDetails(new Map());
  setExpanded(new Set<string>());
  setSelectedType(null);
  setSelectedMember(null);
  setCursor(null);
  setSymbolFailure(null);
  setOutlineFailure(null);
  setLoading(true);
  try {
    const read = await runCommand<Outline>(COMMAND.outline, { project });
    if (mine === generation) {
      setOutline(read);
    }
  } catch (error) {
    if (mine === generation) {
      setOutlineFailure(failed(error));
    }
  } finally {
    if (mine === generation) {
      setLoading(false);
    }
  }
}

/** Reads a symbol unless it is known already. Returns it, or `null` if the read failed or the
 * project changed meanwhile. */
export async function loadSymbol(project: number, id: number): Promise<SymbolInfo | null> {
  const known = details().get(id);
  if (known) {
    return known;
  }
  const mine = generation;
  try {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol: id });
    if (mine !== generation) {
      return null;
    }
    setDetails((previous) => new Map(previous).set(id, info));
    setSymbolFailure(null);
    return info;
  } catch (error) {
    if (mine === generation) {
      setSymbolFailure(failed(error));
    }
    return null;
  }
}

/** Opens a row if it is closed, closes it if it is open. Returns whether it is open now. */
export function toggleRow(key: string): boolean {
  const open = !expanded().has(key);
  setExpanded((previous) => {
    const next = new Set(previous);
    if (open) {
      next.add(key);
    } else {
      next.delete(key);
    }
    return next;
  });
  return open;
}

/** Opens rows. */
export function openRows(keys: readonly string[]): void {
  setExpanded((previous) => new Set([...previous, ...keys]));
}

/** Closes every row. */
export function collapseAll(): void {
  setExpanded(new Set<string>());
}

/** Shows a type in the middle column. The open rows stay as they are: a selection from inside the
 * tree never opens or closes anything (blueprint 15.4). */
export async function selectType(
  project: number,
  id: number,
  member: number | null = null,
): Promise<void> {
  setSelectedType(id);
  setSelectedMember(member);
  setSymbolFailure(null);
  await loadSymbol(project, id);
}

/** Opens or closes a type row; an opened type is read, so its groups can show. */
export async function toggleType(project: number, id: number): Promise<void> {
  if (toggleRow(typeKey(id))) {
    await loadSymbol(project, id);
  }
}

/** Shows a type that was reached from outside the tree (a link in the member table, the
 * selection reveal): its parent type opens its group of nested types, so the row is visible. */
export async function revealNested(project: number, parent: number, id: number): Promise<void> {
  openRows([typeKey(parent), groupKey(parent, GROUP.nested)]);
  await selectType(project, id);
}

/** Goes where the pending navigation asks: opens the rows down to the type (and the group of the
 * member), shows the type and puts the cursor on the row. External navigation opens nodes
 * (blueprint 15.4). */
export async function applyFocus(project: number, tree: Outline): Promise<void> {
  const target = pending();
  if (target === null) {
    return;
  }
  setPending(null);
  const group = target.kind === null ? null : groupOfKind(target.kind);
  openRows([
    ...revealKeys(tree, target.type),
    ...(target.member !== null && group !== null ? [groupKey(target.type, group)] : []),
  ]);
  setCursor(target.member === null ? typeKey(target.type) : memberKey(target.member));
  await selectType(project, target.type, target.member);
}
