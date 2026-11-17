import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { pendingFocus, requestFocus } from "../../shared/navigation/navigation";
import { CHIP, COMMAND, GROUP, NOT_FOUND, TYPE_KINDS, type ChipId } from "./consts";
import { failed, type Failure } from "./failure";
import { INITIAL_SORT, tableRows, type MemberSort } from "./memberSort";
import { groupKey, groupOfKind, memberKey, revealKeys, typeKey } from "./tree";
import type { MemberRow, Outline, SymbolInfo } from "./types";

export type { Failure };

const [outline, setOutline] = createSignal<Outline | null>(null);
const [outlineFailure, setOutlineFailure] = createSignal<Failure | null>(null);
const [symbolFailure, setSymbolFailure] = createSignal<Failure | null>(null);
const [loading, setLoading] = createSignal(false);
const [details, setDetails] = createSignal<ReadonlyMap<number, SymbolInfo>>(new Map());
const [expanded, setExpanded] = createSignal<ReadonlySet<string>>(new Set<string>());
const [selectedType, setSelectedType] = createSignal<number | null>(null);
const [selectedMember, setSelectedMember] = createSignal<number | null>(null);
const [fromTable, setFromTable] = createSignal(false);
const [filter, setFilter] = createSignal("");
const [cursor, setCursor] = createSignal<string | null>(null);
const [sort, setSort] = createSignal<MemberSort | null>(INITIAL_SORT);
const [chip, setChip] = createSignal<ChipId>(CHIP.all);

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
/** Whether the shown member was picked in the member table, so a way back to the table makes
 * sense; a member reached through the tree or by a key has none. */
export const memberFromTable = fromTable;
/** The row of the tree that the keys act on. */
export const treeCursor = cursor;
export const setTreeCursor = setCursor;
/** How the member table is sorted. It stays when another type is shown. */
export const memberSort = sort;
export const setMemberSort = setSort;
/** Which members the table shows (all, properties, methods, public only). */
export const memberChip = chip;
export const setMemberChip = setChip;
/** What the filter field holds. */
export const treeFilter = filter;
export const setTreeFilter = setFilter;

/** The project whose tree is held, `null` if none was read yet. */
let heldProject: number | null = null;

/** The scroll position of the tree, kept so that it is the same when the screen comes back. */
const [scroll, setScroll] = createSignal(0);
export const treeScroll = scroll;
export const setTreeScroll = setScroll;

/** Forgets the held tree, so the next visit reads it from nothing (the tests start so). */
export function forgetExplorer(): void {
  heldProject = null;
}

/** Shows the tree of `project`. If the screen comes back to the project it showed before, the open
 * rows, the selection, the cursor and the scroll position stay and the tree is only read again
 * quietly, so a scan in between shows up. Another project starts from nothing. */
export async function loadOutline(project: number): Promise<void> {
  if (heldProject === project && outline() !== null) {
    await refreshOutline(project);
    return;
  }
  await readOutline(project);
}

/** Reads one symbol again; `null` if it is gone (a scan removed it) or cannot be read. */
async function rereadSymbol(project: number, id: number): Promise<SymbolInfo | null> {
  try {
    return await runCommand<SymbolInfo>(COMMAND.symbol, { project, symbol: id });
  } catch {
    return null;
  }
}

/** Reads the tree and the symbols read so far again, and keeps what the user did with them. A
 * symbol that cannot be read any more is dropped from the held ones; the tree is still replaced. */
async function refreshOutline(project: number): Promise<void> {
  const mine = generation;
  try {
    const read = await runCommand<Outline>(COMMAND.outline, { project });
    const known = [...details().keys()];
    const symbols = await Promise.all(known.map((id) => rereadSymbol(project, id)));
    if (mine === generation) {
      setOutline(read);
      setDetails(
        new Map(
          known.flatMap((id, index) => {
            const info = symbols[index];
            return info ? [[id, info] as const] : [];
          }),
        ),
      );
    }
  } catch {
    // The old tree is still right for what the user sees; the next scan or visit tries again.
  }
}

/** The member row that is selected in the shown type, `null` if no member is selected (or it is
 * not a member of the shown type). The details then show the member, not the type. */
export function shownMember(): MemberRow | null {
  const type = selectedType();
  const id = selectedMember();
  const info = type === null || id === null ? undefined : details().get(type);
  return info?.members.find((member) => member.id === id) ?? null;
}

/** Whether the shown symbol belongs to a layer of the dependency map. A member belongs to the
 * layer of its type. */
export function shownHasLayer(): boolean {
  const type = selectedType();
  return type !== null && (details().get(type)?.layer ?? null) !== null;
}

/** Reads the tree of `project` and forgets everything that was read for another one. */
async function readOutline(project: number): Promise<void> {
  heldProject = project;
  setScroll(0);
  generation += 1;
  const mine = generation;
  setOutline(null);
  setDetails(new Map());
  setExpanded(new Set<string>());
  setSelectedType(null);
  setSelectedMember(null);
  setChip(CHIP.all);
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
  picked = false,
): Promise<void> {
  setSelectedType(id);
  setSelectedMember(member);
  setFromTable(picked && member !== null);
  setSymbolFailure(null);
  await loadSymbol(project, id);
}

/** Moves the selection to the previous (`-1`) or next (`1`) member of the shown type, in the order
 * of the member table; with no member selected it starts at the first or the last one. A nested
 * type is not a member of the source and is skipped. Stops at the ends. */
export async function stepMember(project: number, step: number): Promise<void> {
  const type = selectedType();
  const members = (type === null ? undefined : details().get(type))?.members ?? [];
  const own = tableRows(members, chip(), sort()).filter(
    (member) => !TYPE_KINDS.includes(member.kind),
  );
  if (type === null || own.length === 0) {
    return;
  }
  const at = own.findIndex((member) => member.id === selectedMember());
  const start = step > 0 ? NOT_FOUND : own.length;
  const next = Math.min(own.length - 1, Math.max(0, (at === NOT_FOUND ? start : at) + step));
  await selectType(project, type, own[next]?.id ?? null);
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
  const target = pendingFocus();
  if (target === null) {
    return;
  }
  requestFocus(null);
  setFilter("");
  const group = target.kind === null ? null : groupOfKind(target.kind);
  const owners = await nestedOwners(project, target.type);
  openRows([
    ...revealKeys(tree, owners[0] ?? target.type),
    ...owners.flatMap((owner) => [typeKey(owner), groupKey(owner, GROUP.nested)]),
    typeKey(target.type),
    ...(target.member !== null && group !== null ? [groupKey(target.type, group)] : []),
  ]);
  setCursor(target.member === null ? typeKey(target.type) : memberKey(target.member));
  await selectType(project, target.type, target.member);
}

/** The types that hold a nested type, outermost first; none for a type that stands in a
 * namespace. The tree only lists the types of the namespaces, so the way down to a nested type
 * is read from its breadcrumb, and each holder is read so that its rows can show. */
async function nestedOwners(project: number, type: number): Promise<number[]> {
  const info = await loadSymbol(project, type);
  const owners = (info?.breadcrumb ?? [])
    .filter((crumb) => crumb.id !== type && TYPE_KINDS.includes(crumb.kind))
    .map((crumb) => crumb.id);
  await Promise.all(owners.map((owner) => loadSymbol(project, owner)));
  return owners;
}
