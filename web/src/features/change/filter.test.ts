import { describe, expect, it } from "vitest";
import { FILTER_ALL } from "./consts";
import { isFiltering, kindsOf, NO_FILTER, passes, type RowFacts } from "./filter";

const FACTS: RowFacts = {
  cause: "changed",
  kind: "method",
  name: "Shop.Pricing.Price.Round",
  visibility: "public",
  open: 2,
};

describe("filter of the symbols", () => {
  it("lets everything pass without a choice", () => {
    expect(passes(FACTS, NO_FILTER, true)).toBe(true);
    expect(isFiltering(NO_FILTER, true)).toBe(false);
  });

  it("searches anywhere in the qualified name, ignoring case and blanks around", () => {
    expect(passes(FACTS, { ...NO_FILTER, query: "  pricing.price " }, true)).toBe(true);
    expect(passes(FACTS, { ...NO_FILTER, query: "checkout" }, true)).toBe(false);
  });

  it("combines the kind of change, the kind of symbol and the visibility", () => {
    expect(passes(FACTS, { ...NO_FILTER, cause: "added" }, true)).toBe(false);
    expect(passes(FACTS, { ...NO_FILTER, kind: "class" }, true)).toBe(false);
    expect(passes(FACTS, { ...NO_FILTER, kind: "method", cause: "changed" }, true)).toBe(true);
    expect(passes({ ...FACTS, visibility: null }, { ...NO_FILTER, onlyPublic: true }, true)).toBe(
      false,
    );
  });

  it("counts open places only in the view that has them", () => {
    const closed = { ...FACTS, open: 0 };
    const chosen = { ...NO_FILTER, onlyOpen: true };
    expect(passes(closed, chosen, true)).toBe(false);
    expect(passes(closed, chosen, false)).toBe(true);
    expect(isFiltering(chosen, false)).toBe(false);
    expect(isFiltering(chosen, true)).toBe(true);
  });

  it("lists each kind once, sorted", () => {
    const rows = [FACTS, { ...FACTS, kind: "class" }, FACTS];
    expect(kindsOf(rows)).toEqual(["class", "method"]);
    expect(NO_FILTER.cause).toBe(FILTER_ALL);
  });
});
