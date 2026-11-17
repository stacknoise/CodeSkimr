/** Keys that move along a tab list, and how far each moves. */
const ARROW = { next: "ArrowRight", previous: "ArrowLeft" } as const;
const FORWARD = 1;
const BACK = -1;
const STAY = 0;

/** How far a key moves along the tabs: right is forward, left back, any other key stays. */
function stepOf(key: string): number {
  if (key === ARROW.next) {
    return FORWARD;
  }
  return key === ARROW.previous ? BACK : STAY;
}

/** The pattern of a tab list: an arrow key selects the next or the previous tab, wraps around at
 * the ends and puts the focus on the tab that was chosen (its button has the id `prefix + tab`). */
export function moveAlongTabs<Tab extends string>(
  event: KeyboardEvent,
  tabs: readonly Tab[],
  current: Tab,
  select: (tab: Tab) => void,
  idPrefix: string,
): void {
  const step = stepOf(event.key);
  const next = tabs[(tabs.indexOf(current) + step + tabs.length) % tabs.length];
  if (step !== STAY && next !== undefined) {
    select(next);
    document.getElementById(`${idPrefix}${next}`)?.focus();
    event.preventDefault();
  }
}
