import { runCommand } from "../../shared/api/commands";
import { COMMAND, COUNT_ONLY_LIMIT, MAX_PAGES, PAGE_SIZE, STATE, type AlertState } from "./consts";
import type { Alert, AlertPage, Dismissed } from "./types";

/** The number of open alerts of the project: a call for one row, which carries the number. */
export async function readOpenCount(project: number): Promise<number> {
  const page = await runCommand<AlertPage>(COMMAND.list, {
    project,
    state: STATE.open,
    limit: COUNT_ONLY_LIMIT,
  });
  return page.open;
}

/** Every alert in a state, page by page, and the number of open alerts. */
export async function readAlerts(
  project: number,
  state: AlertState,
): Promise<{ items: Alert[]; open: number }> {
  const items: Alert[] = [];
  let open = 0;
  let offset: number | null = 0;
  for (let count = 0; offset !== null && count < MAX_PAGES; count += 1) {
    const page: AlertPage = await runCommand<AlertPage>(COMMAND.list, {
      project,
      state,
      offset,
      limit: PAGE_SIZE,
    });
    items.push(...page.items);
    open = page.open;
    offset = page.next_offset;
  }
  return { items, open };
}

/** Dismisses one alert. */
export function dismissAlert(project: number, alert: number): Promise<Dismissed> {
  return runCommand<Dismissed>(COMMAND.dismiss, { project, alert });
}

/** Dismisses every open alert of the kinds; the server wants the confirmation. */
export function dismissAllAlerts(project: number, kinds: readonly string[]): Promise<Dismissed> {
  return runCommand<Dismissed>(COMMAND.dismissAll, { project, kinds, confirm: true });
}
