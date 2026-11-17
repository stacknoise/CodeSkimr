import { activeProject } from "../../shared/project/active";
import { KIND } from "../annotations/consts";
import { compose, toggleCritical } from "../annotations/panel";
import { annotationsAvailable } from "../annotations/store";
import type { Target } from "../annotations/types";
import { currentType, shownMember, symbolDetails } from "./explorer";
import type { MemberRow, SymbolInfo } from "./types";

/** The symbol an annotation on a type is about. */
export function typeTarget(info: SymbolInfo): Target {
  return { kind: info.kind, name: info.qualified_name };
}

/** The symbol an annotation on a member is about. */
export function memberTarget(member: MemberRow): Target {
  return { kind: member.kind, name: member.qualified_name };
}

/** The type that the middle column shows, `null` if none is read yet. */
export function shownType(): SymbolInfo | null {
  const type = currentType();
  return type === null ? null : (symbolDetails().get(type) ?? null);
}

/** The symbol that the keys `K`, `N` and `T` act on: the selected member, else the shown type. */
export function shownTarget(): Target | null {
  const info = shownType();
  const member = shownMember();
  if (info === null) {
    return null;
  }
  return member === null ? typeTarget(info) : memberTarget(member);
}

/** Runs `action` for the open project and the selected symbol, if there are both and the server
 * has annotations (the static export has none). */
function onSelection(action: (project: number, target: Target) => void): void {
  const project = activeProject();
  const target = shownTarget();
  if (project !== null && target !== null && annotationsAvailable() !== false) {
    action(project, target);
  }
}

/** The key `K`: marks the selection as system critical, or takes the mark off. */
export function toggleSelectionCritical(): void {
  onSelection((project, target) => void toggleCritical(project, target));
}

/** The key `N`: opens the form of a new note for the selection. */
export function noteOnSelection(): void {
  onSelection(() => compose(KIND.note));
}

/** The key `T`: opens the form of a new task for the selection. */
export function taskOnSelection(): void {
  onSelection(() => compose(KIND.task));
}
