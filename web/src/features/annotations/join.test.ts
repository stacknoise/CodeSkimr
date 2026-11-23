import { describe, expect, it } from "vitest";
import { SUMMARY } from "../../tests/annotations";
import { buildIndex, countersOf, criticalBelow, criticalHidden, hasCounters } from "./join";
import type { SummaryRow } from "./types";

const row = (name: string, over: Partial<SummaryRow> = {}): SummaryRow => ({
  target: { kind: "method", name },
  critical: false,
  notes: 0,
  tasks_done: 0,
  tasks_total: 0,
  without_target: false,
  ...over,
});

describe("buildIndex", () => {
  it("joins the rows by kind and qualified name", () => {
    const index = buildIndex(SUMMARY);
    expect(countersOf(index, { kind: "class", name: "Shop.Domain.Order" })).toEqual({
      critical: true,
      notes: 2,
      tasksDone: 1,
      tasksTotal: 3,
    });
    expect(countersOf(index, { kind: "method", name: "Shop.Domain.Order" })).toBeUndefined();
    expect(countersOf(index, null)).toBeUndefined();
  });

  it("adds up the rows of overloads that share a name", () => {
    const index = buildIndex([row("A.f", { notes: 1 }), row("A.f", { notes: 2, critical: true })]);
    expect(countersOf(index, { kind: "method", name: "A.f" })).toMatchObject({
      critical: true,
      notes: 3,
    });
  });

  it("counts two overloads apart when the annotations name their parameters", () => {
    const named = (parameters: string[], over: Partial<SummaryRow>): SummaryRow =>
      row("A.f", { ...over, target: { kind: "method", name: "A.f", parameters } });
    const index = buildIndex([
      named(["int x"], { notes: 1 }),
      named(["int x", "string s"], { critical: true }),
    ]);
    const short = countersOf(index, { kind: "method", name: "A.f", parameters: ["int x"] });
    const long = countersOf(index, {
      kind: "method",
      name: "A.f",
      parameters: ["int x", "string s"],
    });
    expect(short).toMatchObject({ critical: false, notes: 1 });
    expect(long).toMatchObject({ critical: true, notes: 0 });
  });

  it("falls back to an annotation without parameters, made before overloads were told apart", () => {
    const index = buildIndex([row("A.f", { notes: 4 })]);
    const target = { kind: "method", name: "A.f", parameters: ["int x"] };
    expect(countersOf(index, target)).toMatchObject({ notes: 4 });
  });

  it("leaves out a symbol that is gone from the snapshot", () => {
    const index = buildIndex([row("A.g", { critical: true, without_target: true })]);
    expect(index.exact.size).toBe(0);
    expect(index.critical).toEqual([]);
  });

  it("finds a symbol by its name alone, for the dependency map", () => {
    expect(buildIndex(SUMMARY).byName.get("Shop.Domain.Order")?.critical).toBe(true);
  });
});

describe("critical symbols below a row", () => {
  const index = buildIndex([
    row("Shop", { critical: true }),
    row("Shop.Domain.Order", { critical: true }),
    row("Shop.Domain.Order.Total", { critical: true }),
    row("Shop.DomainX.Other", { critical: true }),
    row("Shop::Rust::Thing", { critical: true }),
    row("Shop.Domain.Order.Id"),
  ]);

  it("counts what continues the name after a dot or a double colon, not the symbol itself", () => {
    expect(criticalBelow(index, "Shop.Domain")).toBe(2);
    expect(criticalBelow(index, "Shop")).toBe(4);
    expect(criticalBelow(index, "Shop.Domain.Order")).toBe(1);
    expect(criticalBelow(index, "Nothing")).toBe(0);
  });

  it("counts the critical ones among the members of a group", () => {
    expect(criticalHidden(index, null, ["Shop.Domain.Order.Total", "Shop.Domain.Order.Id"])).toBe(
      1,
    );
    expect(criticalHidden(index, "Shop.Domain", [])).toBe(2);
  });
});

describe("hasCounters", () => {
  it("is true for a symbol with a note or a task", () => {
    expect(hasCounters(undefined)).toBe(false);
    expect(hasCounters({ critical: true, notes: 0, tasksDone: 0, tasksTotal: 0 })).toBe(false);
    expect(hasCounters({ critical: false, notes: 0, tasksDone: 0, tasksTotal: 1 })).toBe(true);
  });
});
