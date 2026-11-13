import { describe, expect, it } from "vitest";
import { ApiError } from "../shared/api/client";
import type { UsagesAnswer } from "../features/usages/types";
import { DATA, FOCUS_ANSWER as FOCUS, SUMMARY } from "../tests/export-data";
import { answer, installData, runCommand } from "./commands";
import { searchSymbols } from "./search";

const ASK = { symbol: 1, way: "both", depth: 0, include_tests: true };

function usages(input: Partial<typeof ASK>): UsagesAnswer {
  return answer(DATA, "usages.query", { ...ASK, ...input }) as UsagesAnswer;
}

describe("the commands of the export", () => {
  it("answer the project list, the tree and the map from the data", () => {
    expect(answer(DATA, "projects.list", undefined)).toMatchObject([{ id: 9, name: "demo" }]);
    expect(answer(DATA, "explorer.outline", { project: 9 })).toBe(DATA.outline);
    expect(answer(DATA, "graph.layers", { project: 9 })).toBe(DATA.graph);
  });

  it("name no folder in the project list", () => {
    const rows = answer(DATA, "projects.list", undefined) as { location: string }[];
    expect(rows[0]?.location).toBe("");
  });

  it("never hold source text: the lines are not there and the answer says why", () => {
    const source = answer(DATA, "explorer.source", { symbol: 1 });
    expect(source).toMatchObject({ available: false, excluded: true, lines: [], path: "a.rs" });
  });

  it("refuse an unknown symbol and an unknown command with a 404", () => {
    expect(() => answer(DATA, "explorer.symbol", { symbol: 77 })).toThrow(ApiError);
    expect(() => answer(DATA, "projects.add", { source: "x" })).toThrow(ApiError);
  });

  it("reject without data, and answer once data is installed", async () => {
    await expect(runCommand("graph.layers")).rejects.toBeInstanceOf(ApiError);
    installData(DATA);
    await expect(runCommand("graph.layers")).resolves.toBe(DATA.graph);
  });
});

describe("the usages of the export", () => {
  it("keep the verification of every hit as it was exported", () => {
    const all = usages({});
    expect(all.incoming?.hits.every((found) => !found.verified)).toBe(true);
    expect(all.level2).toBe("unavailable");
    expect(all.summary.unchecked).toBe(3);
  });

  it("narrow the walk to the depth of the toolbar", () => {
    const one = usages({ depth: 1 });
    expect(one.depth).toBe(1);
    expect(one.incoming?.hits.map((found) => found.depth)).toEqual([1, 1]);
    expect(one.incoming?.total).toBe(2);
    expect(usages({ depth: 9 }).depth).toBe(3);
  });

  it("leave out tests together with their share of the counts", () => {
    const without = usages({ include_tests: false });
    expect(without.incoming?.hits.some((found) => found.kind === "test")).toBe(false);
    expect(without.summary.places).toBe(1);
  });

  it("leave out the direction that is not asked for", () => {
    expect(usages({ way: "incoming" }).outgoing).toBeNull();
    expect(usages({ way: "outgoing" }).incoming).toBeNull();
  });

  it("refuse a symbol without usages", () => {
    expect(() => answer(DATA, "usages.query", { ...ASK, symbol: 2 })).toThrow(ApiError);
  });
});

describe("the search of the export", () => {
  it("finds a name by its start before a name that only contains the text", () => {
    const names = searchSymbols(DATA, "fo").map((found) => found.name);
    expect(names[0]).toBe("Foo");
    expect(names).toContain("frobnicate");
  });

  it("finds a name by letters in order and names the type that holds a member", () => {
    const found = searchSymbols(DATA, "frbn");
    expect(found.map((one) => one.name)).toEqual(["frobnicate"]);
    expect(found[0]?.parent).toBe(1);
  });

  it("finds nothing for an empty text", () => {
    expect(searchSymbols(DATA, "  ")).toEqual([]);
  });
});
