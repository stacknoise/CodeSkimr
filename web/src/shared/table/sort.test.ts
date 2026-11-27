import { describe, expect, it } from "vitest";
import { SORT_DIRECTION } from "./consts";
import { compareValues, pressSort } from "./sort";

describe("pressSort", () => {
  it("starts a text column from A and a number column with the largest", () => {
    expect(pressSort(null, "name", false)).toEqual({
      column: "name",
      direction: SORT_DIRECTION.ascending,
    });
    expect(pressSort(null, "size", true)?.direction).toBe(SORT_DIRECTION.descending);
  });

  it("turns the same column around once and then removes the sort", () => {
    const first = pressSort(null, "name", false);
    const turned = pressSort(first, "name", false);
    expect(turned?.direction).toBe(SORT_DIRECTION.descending);
    expect(pressSort(turned, "name", false)).toBeNull();
  });
});

describe("compareValues", () => {
  it("sorts texts by the language and numbers by size", () => {
    expect(compareValues("Äpfel", "beta", SORT_DIRECTION.ascending, "de")).toBeLessThan(0);
    expect(compareValues(2, 10, SORT_DIRECTION.ascending, "en")).toBeLessThan(0);
    expect(compareValues(2, 10, SORT_DIRECTION.descending, "en")).toBeGreaterThan(0);
  });

  it("puts a missing value last in either direction", () => {
    expect(compareValues(null, 1, SORT_DIRECTION.ascending, "en")).toBeGreaterThan(0);
    expect(compareValues(null, 1, SORT_DIRECTION.descending, "en")).toBeGreaterThan(0);
  });
});
