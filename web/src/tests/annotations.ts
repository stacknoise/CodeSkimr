import type { Annotation, Page, SummaryRow } from "../features/annotations/types";
import { answer, fetchMock } from "./commands";

/** A page that holds all of `items` (the server's answer for a short list). */
export function page<Item>(items: Item[], next: number | null = null): Page<Item> {
  return { items, total: items.length, offset: 0, limit: 500, next_offset: next };
}

/** The order type is critical with two notes and one of three tasks done; its `Total` method is
 * critical with one note and one open task. */
export const SUMMARY: SummaryRow[] = [
  {
    target: { kind: "class", name: "Shop.Domain.Order" },
    critical: true,
    notes: 2,
    tasks_done: 1,
    tasks_total: 3,
    without_target: false,
  },
  {
    target: { kind: "method", name: "Shop.Domain.Order.Total" },
    critical: true,
    notes: 1,
    tasks_done: 0,
    tasks_total: 1,
    without_target: false,
  },
];

const base = {
  project: 1,
  target: { kind: "class", name: "Shop.Domain.Order" },
  author: "Anton",
  created: 1_794_000_000,
  updated: 1_794_000_000,
  without_target: false,
};

export const MARK: Annotation = { ...base, id: 1, kind: "critical", text: "Money moves here." };
export const NOTE: Annotation = {
  ...base,
  id: 2,
  kind: "note",
  text: "Discount before **tax**. See https://example.org/wiki and [docs](javascript:alert(1)).",
};
export const TASK: Annotation = {
  ...base,
  id: 3,
  kind: "task",
  title: "Add a rounding test",
  status: "open",
  due: 1_795_000_000,
  assignee: "Anna",
};
export const DONE_TASK: Annotation = {
  ...base,
  id: 4,
  kind: "task",
  title: "Remove the double calculation",
  status: "done",
};

/** The bodies of the calls of one command, in order. */
export function bodiesOf(id: string): Record<string, unknown>[] {
  return fetchMock.mock.calls
    .filter(([path]) => String(path).endsWith(`/${id}`))
    .map(([, init]) => JSON.parse(String((init as RequestInit).body ?? "{}")));
}

/** How often a command was called. */
export function callsOf(id: string): number {
  return bodiesOf(id).length;
}

/** Answers the command calls by id like `serve`, with the body of the call for the handler. A
 * command that has no handler is a 404. */
export function serveBodies(handlers: Record<string, (body: Record<string, unknown>) => Response>) {
  fetchMock.mockImplementation((path: string, init?: RequestInit) => {
    const id = path.replace("/api/v1/commands/", "");
    const body = JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>;
    return Promise.resolve(handlers[id]?.(body) ?? answer({ code: "API-201" }, 404));
  });
}
