import type { Alert, AlertPage } from "../features/alerts/types";

const NOW = Math.floor(Date.now() / 1000);

const base: Alert = {
  id: 1,
  annotation: 10,
  target: { kind: "method", name: "Shop.Orders.OrderService.RecalculateTotal" },
  reason: "Moves money.",
  kind: "changed",
  state: "open",
  counter: 1,
  first_seen: NOW - 180,
  last_seen: NOW - 180,
  dismissed: null,
  snapshot: 5,
  commit: null,
  trigger: "watcher",
  changes: ["cc", "loc"],
  references: null,
  closest: null,
};

/** A changed method, a removed method with four references, an orphaned target with a closest
 * symbol and one that was dismissed. */
export const CHANGED: Alert = base;
export const REMOVED: Alert = {
  ...base,
  id: 2,
  annotation: 11,
  target: { kind: "method", name: "Shop.Orders.IOrderRepository.Save" },
  kind: "removed",
  references: 4,
  changes: [],
};
export const ORPHANED: Alert = {
  ...base,
  id: 3,
  annotation: 12,
  target: { kind: "method", name: "Shop.Orders.OrderService.Recalc" },
  kind: "orphaned",
  closest: "Shop.Orders.OrderService.RecalculateTotal",
  changes: [],
};
export const DISMISSED: Alert = {
  ...base,
  id: 4,
  annotation: 13,
  state: "dismissed",
  dismissed: NOW - 60,
};

/** The answer of `alerts.list` for the alerts of one state. */
export function alertPage(items: Alert[], open: number): AlertPage {
  return { items, total: items.length, offset: 0, limit: 500, next_offset: null, open };
}
