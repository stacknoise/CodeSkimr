import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { COMMAND } from "./consts";
import { typeKey } from "./tree";
import type { Outline, SymbolInfo } from "./types";

/** Why a read failed: a registered code, or `null` if the server could not be reached. */
export type Failure = { code: ErrorCode | null };

const [outline, setOutline] = createSignal<Outline | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [details, setDetails] = createSignal<ReadonlyMap<number, SymbolInfo>>(new Map());
const [expanded, setExpanded] = createSignal<ReadonlySet<string>>(new Set<string>());
const [selectedType, setSelectedType] = createSignal<number | null>(null);
const [selectedMember, setSelectedMember] = createSignal<number | null>(null);
const [filter, setFilter] = createSignal("");

/** The tree of the project, `null` until it has been read. */
export const treeOutline = outline;
/** The failure of the last read, `null` if it worked. */
export const explorerFailure = failure;
/** The symbols read so far, by id. */
export const symbolDetails = details;
/** Keys of the opened rows. */
export const expandedRows = expanded;
/** The type that the middle column shows. */
export const currentType = selectedType;
/** The member that is highlighted in the tree and in the table. */
export const currentMember = selectedMember;
/** What the filter field holds. */
export const treeFilter = filter;
export const setTreeFilter = setFilter;

function failed(error: unknown): Failure {
  return { code: error instanceof ApiError ? error.code : null };
}

/** Reads the tree of `project` and forgets everything that was read for another one. */
export async function loadOutline(project: number): Promise<void> {
  setOutline(null);
  setDetails(new Map());
  setExpanded(new Set<string>());
  setSelectedType(null);
  setSelectedMember(null);
  try {
    setOutline(await runCommand<Outline>(COMMAND.outline, { project }));
    setFailure(null);
  } catch (error) {
    setFailure(failed(error));
  }
}

/** Reads a symbol unless it is known already. Returns it, or `null` if the read failed. */
export async function loadSymbol(project: number, id: number): Promise<SymbolInfo | null> {
  const known = details().get(id);
  if (known) {
    return known;
  }
  try {
    const info = await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol: id });
    setDetails((previous) => new Map(previous).set(id, info));
    setFailure(null);
    return info;
  } catch (error) {
    setFailure(failed(error));
    return null;
  }
}

/** Opens a row if it is closed, closes it if it is open. */
export function toggleRow(key: string): void {
  setExpanded((previous) => {
    const next = new Set(previous);
    if (!next.delete(key)) {
      next.add(key);
    }
    return next;
  });
}

/** Opens one row. */
export function openRow(key: string): void {
  setExpanded((previous) => new Set(previous).add(key));
}

/** Closes every row. */
export function collapseAll(): void {
  setExpanded(new Set<string>());
}

/** Shows a type in the middle column; its row opens into the member groups. */
export async function selectType(
  project: number,
  id: number,
  member: number | null = null,
): Promise<void> {
  setSelectedType(id);
  setSelectedMember(member);
  openRow(typeKey(id));
  await loadSymbol(project, id);
}
