import { createSignal } from "solid-js";

/** The projects that are being removed. The state lives outside the row: while a scan runs the
 * list is read again and the row is built anew. */
const [removing, setRemoving] = createSignal<ReadonlySet<number>>(new Set());

/** Whether the project is being removed: its row is locked then. */
export function isRemoving(id: number): boolean {
  return removing().has(id);
}

/** Whether the row is being removed: by the removal of this tab, or as the server says (a removal
 * that began in another tab, or before this screen was left and shown again). */
export function removingNow(row: { id: number; removing?: boolean }): boolean {
  return row.removing === true || isRemoving(row.id);
}

/** Whether the removal of the row is stopped half way and waits to be asked for again. */
export function interruptedNow(row: { id: number; interrupted?: boolean }): boolean {
  return row.interrupted === true && !isRemoving(row.id);
}

/** Marks a project as being removed, or as done with it. */
export function markRemoving(id: number, on: boolean): void {
  setRemoving((previous) => {
    const next = new Set(previous);
    if (on) {
      next.add(id);
    } else {
      next.delete(id);
    }
    return next;
  });
}
