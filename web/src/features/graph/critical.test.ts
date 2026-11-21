import { describe, expect, it } from "vitest";
import { SUMMARY } from "../../tests/annotations";
import { buildIndex } from "../annotations/join";
import { hasCriticalBox, isCriticalBox, narrowToCritical } from "./critical";
import { LEVEL } from "./consts";
import type { GraphData, LayerBox } from "./types";

const box = (name: string, over: Partial<LayerBox> = {}): LayerBox => ({
  name,
  label: name,
  context: "",
  x: 0,
  y: 0,
  width: 100,
  height: 40,
  rank: 0,
  order: 0,
  layer: null,
  types: null,
  symbol: 1,
  ...over,
});

const link = (from: string, to: string) => ({
  from,
  to,
  count: 1,
  violates_layering: false,
  in_cycle: false,
});

const MAP: GraphData = {
  snapshot: 1,
  width: 400,
  height: 300,
  focus: null,
  bands: [],
  boxes: [
    box("Shop.Domain.Order"),
    box("Shop.Domain.Tax"),
    box("Shop.Domain.Far"),
    box("Shop.Domain.Price"),
  ],
  links: [
    link("Shop.Domain.Order", "Shop.Domain.Tax"),
    link("Shop.Domain.Price", "Shop.Domain.Order"),
    link("Shop.Domain.Tax", "Shop.Domain.Far"),
  ],
};

const INDEX = buildIndex(SUMMARY);

describe("isCriticalBox", () => {
  it("knows a critical type by its qualified name", () => {
    expect(isCriticalBox(INDEX, box("Shop.Domain.Order"), LEVEL.types)).toBe(true);
    expect(isCriticalBox(INDEX, box("Shop.Domain.Tax"), LEVEL.types)).toBe(false);
  });

  it("ignores the id that tells two types of one name apart", () => {
    expect(isCriticalBox(INDEX, box("Shop.Domain.Order #7"), LEVEL.types)).toBe(true);
  });

  it("knows no critical layer, and a namespace only as a namespace", () => {
    expect(isCriticalBox(INDEX, box("Shop.Domain.Order"), LEVEL.layers)).toBe(false);
    expect(isCriticalBox(INDEX, box("Shop.Domain.Order"), LEVEL.namespaces)).toBe(false);
    const spaces = buildIndex([
      { ...SUMMARY[0]!, target: { kind: "namespace", name: "Shop.Domain" } },
    ]);
    expect(isCriticalBox(spaces, box("Shop.Domain", { symbol: null }), LEVEL.namespaces)).toBe(
      true,
    );
  });
});

describe("narrowToCritical", () => {
  it("keeps the critical boxes and their neighbours, and the links that touch a critical box", () => {
    const narrow = narrowToCritical(MAP, INDEX, LEVEL.types);
    expect(narrow.boxes.map((one) => one.name)).toEqual([
      "Shop.Domain.Order",
      "Shop.Domain.Tax",
      "Shop.Domain.Price",
    ]);
    expect(narrow.links).toHaveLength(2);
  });

  it("returns the map as it is when no box is critical", () => {
    expect(narrowToCritical(MAP, buildIndex([]), LEVEL.types)).toBe(MAP);
    expect(hasCriticalBox(MAP, buildIndex([]), LEVEL.types)).toBe(false);
    expect(hasCriticalBox(MAP, INDEX, LEVEL.types)).toBe(true);
  });
});
