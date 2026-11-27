import { describe, expect, it } from "vitest";
import { sortCycles, sortLinkPairs, sortLinks, sortRows } from "./order";
import type { LayerLink, ModuleCycle, ModuleRow } from "./types";

const link = (from: string, to: string): LayerLink => ({
  from,
  to,
  count: 1,
  violates_layering: false,
  in_cycle: false,
});

describe("the order of the lists in the panels", () => {
  it("sorts umlauts with their letter and ignores case in German", () => {
    const sorted = sortLinks(
      [link("a", "Zeta"), link("a", "Äpfel"), link("a", "beta")],
      (l) => l.to,
      "de",
    );
    expect(sorted.map((item) => item.to)).toEqual(["Äpfel", "beta", "Zeta"]);
  });

  it("sorts pairs by the name that leaves, then by the one that is reached", () => {
    const sorted = sortLinkPairs([link("b", "a"), link("a", "z"), link("a", "c")], "en");
    expect(sorted.map((item) => item.from + item.to)).toEqual(["ac", "az", "ba"]);
  });

  it("sorts the table by name", () => {
    const row = (name: string) => ({ name }) as ModuleRow;
    expect(sortRows([row("b"), row("a")], "en", null).map((item) => item.name)).toEqual(["a", "b"]);
  });

  it("sorts the table by the pressed column and puts a module without stability last", () => {
    const row = (name: string, outgoing: number, stability: number | null) =>
      ({ name, outgoing, stability }) as ModuleRow;
    const rows = [row("a", 1, null), row("b", 9, 300), row("c", 5, 800)];
    const by = (column: string, direction: "ascending" | "descending") =>
      sortRows(rows, "en", { column, direction }).map((item) => item.name);
    expect(by("out", "descending")).toEqual(["b", "c", "a"]);
    expect(by("stability", "ascending")).toEqual(["b", "c", "a"]);
    expect(by("stability", "descending")).toEqual(["c", "b", "a"]);
    expect(by("name", "descending")).toEqual(["c", "b", "a"]);
  });

  it("sorts cycles and their edges but keeps the way round one ring", () => {
    const ring = (modules: string[], first: string): ModuleCycle => ({
      modules,
      edges: [
        { from: "z", to: first, count: 1 },
        { from: "a", to: first, count: 1 },
      ],
      resolution: [],
    });
    const sorted = sortCycles([ring(["m", "k"], "k"), ring(["b", "c"], "c")], "en");
    expect(sorted.map((cycle) => cycle.modules)).toEqual([
      ["b", "c"],
      ["m", "k"],
    ]);
    expect(sorted[0]!.edges.map((edge) => edge.from)).toEqual(["a", "z"]);
  });
});
