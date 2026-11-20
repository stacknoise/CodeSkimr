import { describe, expect, it } from "vitest";
import { SORT_COLUMN, SORT_DIRECTION } from "./consts";
import { INITIAL_SORT, pressColumn, sortMembers } from "./memberSort";
import type { MemberRow } from "./types";

const row = (name: string, over: Partial<MemberRow> = {}): MemberRow => ({
  id: 1,
  name,
  qualified_name: name,
  kind: "method",
  visibility: "public",
  signature: null,
  complexity: null,
  complexity_rating: null,
  usages: 0,
  first_line: 1,
  last_line: 1,
  ...over,
});

const names = (rows: MemberRow[]) => rows.map((item) => item.name);

describe("sortMembers", () => {
  const rows = [
    row("beta", { complexity: 3, usages: 5, first_line: 30, visibility: "private" }),
    row("Alpha", { complexity: 10, usages: 1, first_line: 20, signature: "int" }),
    row("gamma", { complexity: null, usages: 9, first_line: 10 }),
  ];

  it("starts with the name, without regard to case", () => {
    expect(names(sortMembers(rows, INITIAL_SORT))).toEqual(["Alpha", "beta", "gamma"]);
  });

  it("sorts numbers and lines, and keeps a row without a value last in both directions", () => {
    const by = (
      column: (typeof SORT_COLUMN)[keyof typeof SORT_COLUMN],
      direction: "ascending" | "descending",
    ) => names(sortMembers(rows, { column, direction }));
    expect(by("complexity", "ascending")).toEqual(["beta", "Alpha", "gamma"]);
    expect(by("complexity", "descending")).toEqual(["Alpha", "beta", "gamma"]);
    expect(by("usages", "descending")).toEqual(["gamma", "beta", "Alpha"]);
    expect(by("lines", "ascending")).toEqual(["gamma", "Alpha", "beta"]);
    expect(by("signature", "ascending")).toEqual(["Alpha", "beta", "gamma"]);
    expect(by("access", "ascending")).toEqual(["beta", "Alpha", "gamma"]);
  });

  it("orders rows that both have no value by name", () => {
    const empty = [row("b"), row("a"), row("c", { complexity: 1 })];
    const sorted = sortMembers(empty, {
      column: SORT_COLUMN.complexity,
      direction: SORT_DIRECTION.descending,
    });
    expect(names(sorted)).toEqual(["c", "a", "b"]);
  });

  it("orders equal values by name and does not change the list it is given", () => {
    const same = [row("b"), row("a")];
    expect(
      names(
        sortMembers(same, { column: SORT_COLUMN.usages, direction: SORT_DIRECTION.descending }),
      ),
    ).toEqual(["a", "b"]);
    expect(names(same)).toEqual(["b", "a"]);
  });
});

describe("pressColumn", () => {
  it("turns the same column round once, then removes the sort", () => {
    const turned = pressColumn(INITIAL_SORT, SORT_COLUMN.name);
    expect(turned?.direction).toBe(SORT_DIRECTION.descending);
    expect(pressColumn(turned, SORT_COLUMN.name)).toBeNull();
    expect(pressColumn(null, SORT_COLUMN.name)?.direction).toBe(SORT_DIRECTION.ascending);
    const byUsages = pressColumn(INITIAL_SORT, SORT_COLUMN.usages);
    expect(byUsages).toEqual({ column: SORT_COLUMN.usages, direction: SORT_DIRECTION.descending });
    expect(pressColumn(byUsages, SORT_COLUMN.usages)?.direction).toBe(SORT_DIRECTION.ascending);
    expect(pressColumn(pressColumn(byUsages, SORT_COLUMN.usages), SORT_COLUMN.usages)).toBeNull();
    expect(pressColumn(INITIAL_SORT, SORT_COLUMN.access)?.direction).toBe(SORT_DIRECTION.ascending);
  });

  it("keeps the rows in the order of the code without a sort", () => {
    const rows = [row("b", { first_line: 9 }), row("a", { first_line: 3 })];
    expect(names(sortMembers(rows, null))).toEqual(["a", "b"]);
  });
});
