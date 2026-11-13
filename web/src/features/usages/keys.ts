import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { KEY, NO_HIT, STEP_NEXT, STEP_PREVIOUS } from "./consts";

/** What the keys of the screen do. */
export interface UsagesKeys {
  /** Moves the selection of the hit list by one step; the first call selects the first hit. */
  step: (direction: number) => void;
  /** Opens the selected hit, or the symbol in focus if no hit is selected, in the explorer. */
  open: () => void;
}

/** The index the selection moves to when it steps from `at` through `count` hits, wrapping. */
export function nextHit(at: number, direction: number, count: number): number {
  if (count === 0) {
    return NO_HIT;
  }
  if (at === NO_HIT) {
    return direction === STEP_NEXT ? 0 : count - 1;
  }
  return (at + direction + count) % count;
}

/** Registers `F8` and `Shift+F8` (next and previous hit) and `Enter` (open in the explorer) while
 * the screen is shown. `Enter` on a button or a field keeps its own meaning. */
export function useUsagesKeys(keys: UsagesKeys): void {
  useKeydown((event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) {
      return;
    }
    if (event.key === KEY.next) {
      keys.step(event.shiftKey ? STEP_PREVIOUS : STEP_NEXT);
    } else if (event.key === KEY.open && !isTyping(event.target) && !isButton(event.target)) {
      keys.open();
    } else {
      return;
    }
    event.preventDefault();
  });
}

function isButton(target: EventTarget | null): boolean {
  return target instanceof HTMLButtonElement;
}
