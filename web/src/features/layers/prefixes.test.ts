import { describe, expect, it } from "vitest";
import { buildTree, initiallyOpen, segmentsOf, visibleRows } from "./prefixes";

const NAMESPACES = [
  { name: "crates::domain", types: 2, layer: null },
  { name: "crates::domain::model", types: 3, layer: null },
  { name: "crates::app", types: 1, layer: null },
  { name: "Shop.Orders", types: 4, layer: null },
];

describe("segmentsOf", () => {
  it("splits at :: and at dots and keeps the prefix of each segment", () => {
    expect(segmentsOf("a::b.c")).toEqual([
      { prefix: "a", label: "a" },
      { prefix: "a::b", label: "b" },
      { prefix: "a::b.c", label: "c" },
    ]);
  });
});

describe("buildTree", () => {
  it("counts the types below every prefix", () => {
    const [crates, shop] = buildTree(NAMESPACES);
    expect(crates?.prefix).toBe("crates");
    expect(crates?.types).toBe(6);
    expect(crates?.children.map((child) => child.prefix)).toEqual([
      "crates::domain",
      "crates::app",
    ]);
    expect(crates?.children[0]?.types).toBe(5);
    expect(shop?.children[0]?.label).toBe("Orders");
  });
});

describe("visibleRows", () => {
  it("shows the children of opened rows only", () => {
    const roots = buildTree(NAMESPACES);
    expect(visibleRows(roots, new Set()).map((row) => row.prefix)).toEqual(["crates", "Shop"]);
    const rows = visibleRows(roots, new Set(["crates", "crates::domain"]));
    expect(rows.map((row) => row.prefix)).toEqual([
      "crates",
      "crates::domain",
      "crates::domain::model",
      "crates::app",
      "Shop",
    ]);
  });

  it("opens small trees at once and big ones not at all", () => {
    expect(initiallyOpen(buildTree(NAMESPACES)).size).toBe(6);
    const big = Array.from({ length: 200 }, (_, at) => ({
      name: `space${at}`,
      types: 1,
      layer: null,
    }));
    expect(initiallyOpen(buildTree(big)).size).toBe(0);
  });
});
