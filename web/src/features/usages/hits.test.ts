import { describe, expect, it } from "vitest";
import { ANSWER, HANDLER, VIEW } from "../../tests/usages";
import { depthsUpTo, listedHits, splitAt } from "./hits";

describe("listedHits", () => {
  it("lists only the direct hits", () => {
    expect(listedHits(ANSWER).map((hit) => hit.from.id)).toEqual([11, 12, 13]);
  });

  it("narrows the list to the symbol picked in the graph", () => {
    expect(listedHits(ANSWER, HANDLER.id).map((hit) => hit.from.id)).toEqual([12]);
    expect(listedHits(ANSWER, VIEW.id).length).toBe(1);
  });
});

describe("splitAt", () => {
  it("marks a whole word, not the start of a longer name", () => {
    const parts = splitAt("return new AcceptedOperation(args.Accept);", "Accept");
    expect(parts.before).toBe("return new AcceptedOperation(args.");
    expect(parts.after).toBe(");");
  });

  it("marks nothing if the word is only part of other names", () => {
    expect(splitAt("AcceptedOperation", "Accept").word).toBe("");
  });
});

describe("depthsUpTo", () => {
  it("offers every depth and `all` when nothing limits the walk", () => {
    expect(depthsUpTo(null)).toEqual([1, 2, 3, 0]);
  });

  it("offers only the steps up to the limit and never `all`", () => {
    expect(depthsUpTo(1)).toEqual([1]);
    expect(depthsUpTo(2)).toEqual([1, 2]);
  });
});
