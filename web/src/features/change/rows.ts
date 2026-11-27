import {
  CAUSE,
  CAUSE_MOVED,
  CAUSE_PILL,
  MOVE_ARROW,
  REASON,
  REASON_PILL,
  ROW_KEY,
  SIGN,
  STATUS,
  type PillSpec,
} from "./consts";
import type {
  ChangeAnswer,
  ImpactRow,
  MatchedOut,
  SymbolImpactOut,
  SymbolOut,
  UsageOut,
} from "./types";

/** The impact of a symbol of the base scan, found by its id and the cause. */
function impactOf(answer: ChangeAnswer, cause: string, base: SymbolOut): SymbolImpactOut | null {
  return (
    answer.impact?.items.find((one) => one.cause === cause && one.symbol.id === base.id) ?? null
  );
}

function removedRow(answer: ChangeAnswer, symbol: SymbolOut): ImpactRow {
  return {
    key: `${CAUSE.removed}:${symbol.id}`,
    cause: CAUSE.removed,
    symbol,
    reasons: [],
    narrowed: false,
    before: null,
    impact: impactOf(answer, CAUSE.removed, symbol),
  };
}

function changedRow(answer: ChangeAnswer, matched: MatchedOut): ImpactRow {
  return {
    key: `${CAUSE.changed}:${matched.target.id}`,
    cause: CAUSE.changed,
    symbol: matched.target,
    reasons: matched.reasons,
    narrowed: matched.narrowed,
    before: matched.base,
    impact: impactOf(answer, CAUSE.changed, matched.base),
  };
}

function addedRow(symbol: SymbolOut): ImpactRow {
  return {
    key: `${CAUSE.added}:${symbol.id}`,
    cause: CAUSE.added,
    symbol,
    reasons: [],
    narrowed: false,
    before: null,
    impact: null,
  };
}

/** The rows of the table of the impact: the removed symbols first, then the changed ones (those
 * with a checklist before those without), then the new ones. */
export function impactRows(answer: ChangeAnswer): ImpactRow[] {
  const changed = answer.changed.items.map((matched) => changedRow(answer, matched));
  const withChecklist = changed.filter((row) => row.impact !== null);
  const without = changed.filter((row) => row.impact === null);
  return [
    ...answer.removed.items.map((symbol) => removedRow(answer, symbol)),
    ...withChecklist,
    ...without,
    ...answer.added.items.map(addedRow),
  ];
}

/** How many usages of the row are not touched: the places that are still to check. */
export function openCount(row: ImpactRow): number {
  return row.impact?.not_touched ?? 0;
}

/** The usages of all rows that the answer lists. */
function allUsages(answer: ChangeAnswer): UsageOut[] {
  return (answer.impact?.items ?? []).flatMap((one) => one.usages.items);
}

/** How many different files hold a listed usage. */
export function affectedFiles(answer: ChangeAnswer): number {
  return new Set(allUsages(answer).map((usage) => usage.path)).size;
}

/** The numbers of the symbols of a difference: new, changed, removed. */
export interface SymbolCounts {
  added: number;
  changed: number;
  removed: number;
}

/** How many symbols the difference adds, changes and removes. */
export function symbolCounts(answer: ChangeAnswer): SymbolCounts {
  return {
    added: answer.added.total,
    changed: answer.changed.total,
    removed: answer.removed.total,
  };
}

/** How many of the changed symbols changed for a reason. */
export function reasonCount(answer: ChangeAnswer, reason: string): number {
  return answer.changed.items.filter((matched) => matched.reasons.includes(reason)).length;
}

/** A line of the picture of a changed signature: its sign and its text. */
export interface DiffLine {
  sign: string;
  text: string;
}

/** The text a symbol is shown with: its signature, or its name if it has none. */
function signatureOf(symbol: SymbolOut): string {
  return symbol.signature ?? symbol.qualified_name;
}

/** The lines that show what changed: the old and the new signature, the old and the new
 * visibility, or one line for a symbol that is gone or new. A change of the content alone has
 * none. */
export function diffLines(row: ImpactRow): DiffLine[] {
  if (row.cause === CAUSE.removed) {
    return [{ sign: SIGN.before, text: signatureOf(row.symbol) }];
  }
  if (row.cause === CAUSE.added) {
    return [{ sign: SIGN.after, text: signatureOf(row.symbol) }];
  }
  const before = row.before;
  const lines: DiffLine[] = [];
  if (before !== null && row.reasons.includes(REASON.signature)) {
    lines.push({ sign: SIGN.before, text: signatureOf(before) });
    lines.push({ sign: SIGN.after, text: signatureOf(row.symbol) });
  }
  const wasVisible = before?.visibility ?? null;
  if (wasVisible !== null && row.reasons.includes(REASON.visibility)) {
    lines.push({ sign: SIGN.before, text: wasVisible });
    lines.push({ sign: SIGN.after, text: row.symbol.visibility ?? wasVisible });
  }
  return lines;
}

/** Usages of one file, in the order the answer has them. */
export interface UsageGroup {
  path: string;
  open: number;
  usages: UsageOut[];
}

/** Groups usages by file; `onlyOpen` leaves out the ones that were adapted. */
export function groupUsages(usages: readonly UsageOut[], onlyOpen: boolean): UsageGroup[] {
  const groups = new Map<string, UsageGroup>();
  for (const usage of usages) {
    if (onlyOpen && usage.status !== STATUS.notTouched) {
      continue;
    }
    const group = groups.get(usage.path) ?? { path: usage.path, open: 0, usages: [] };
    group.usages.push(usage);
    group.open += usage.status === STATUS.notTouched ? 1 : 0;
    groups.set(usage.path, group);
  }
  return [...groups.values()];
}

/** One row of the list of the comparison: the pills that say what happened, the symbol and where
 * it lives. */
export interface CompareRow {
  key: string;
  pills: PillSpec[];
  name: string;
  signature: string | null;
  where: string;
}

function reasonPills(reasons: readonly string[]): PillSpec[] {
  return reasons.flatMap((reason) => REASON_PILL[reason] ?? []);
}

/** The pill for what happened to a symbol, as a list of at most one. */
function causePill(cause: string): PillSpec[] {
  const pill = CAUSE_PILL[cause];
  return pill === undefined ? [] : [pill];
}

/** The pills that name a row of the impact: the reasons of a change, or what else happened. */
export function pillsOf(row: ImpactRow): PillSpec[] {
  return row.cause === CAUSE.changed ? reasonPills(row.reasons) : causePill(row.cause);
}

type PlainCause = typeof CAUSE.removed | typeof CAUSE.added;
type MatchedCause = typeof CAUSE.changed | typeof CAUSE_MOVED;

function plainRow(cause: PlainCause, symbol: SymbolOut): CompareRow {
  return {
    key: `${cause}:${symbol.id}`,
    pills: causePill(cause),
    name: symbol.qualified_name,
    signature: symbol.signature,
    where: symbol.path,
  };
}

function matchedRow(kind: MatchedCause, matched: MatchedOut): CompareRow {
  const moved = kind === CAUSE_MOVED;
  return {
    key: `${kind}:${matched.target.id}`,
    pills: [...(moved ? causePill(CAUSE_MOVED) : []), ...reasonPills(matched.reasons)],
    name: matched.target.qualified_name,
    signature: matched.target.signature,
    where: moved ? `${matched.base.path}${MOVE_ARROW}${matched.target.path}` : matched.target.path,
  };
}

/** The rows of the list of the comparison, in the order of the mock: removed, changed, moved, new. */
export function compareRows(answer: ChangeAnswer): CompareRow[] {
  return [
    ...answer.removed.items.map((symbol) => plainRow(CAUSE.removed, symbol)),
    ...answer.changed.items.map((matched) => matchedRow(CAUSE.changed, matched)),
    ...answer.moved.items.map((matched) => matchedRow(CAUSE_MOVED, matched)),
    ...answer.added.items.map((symbol) => plainRow(CAUSE.added, symbol)),
  ];
}

/** The row that a key of the table selects, counted from the row that is selected now: the arrows
 * move by one and stop at the ends, Home and End go to the ends. `null` for any other key. */
export function nextRow(key: string, index: number, count: number): number | null {
  const last = count - 1;
  const moves: Record<string, number> = {
    [ROW_KEY.next]: Math.min(index + 1, last),
    [ROW_KEY.previous]: Math.max(index - 1, 0),
    [ROW_KEY.first]: 0,
    [ROW_KEY.last]: last,
  };
  return count === 0 ? null : (moves[key] ?? null);
}
