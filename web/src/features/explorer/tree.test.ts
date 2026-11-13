import { describe, expect, it } from "vitest";
import { buildRows, layerKey, namespaceKey, revealKeys, typeKey, type TreeLabels } from "./tree";
import type { Outline, SymbolInfo } from "./types";

const LABELS: TreeLabels = {
  noLayer: "none",
  noNamespace: "root",
  fields: "Fields",
  methods: "Methods",
};

const OUTLINE: Outline = {
  snapshot: 1,
  layers: [
    {
      name: "Domain",
      namespaces: [
        {
          name: "App.Domain",
          types: [
            { id: 1, name: "Order", kind: "class", members: 2 },
            { id: 2, name: "IOrder", kind: "interface", members: 0 },
          ],
        },
      ],
    },
    {
      name: "",
      namespaces: [{ name: "", types: [{ id: 3, name: "Loose", kind: "struct", members: 0 }] }],
    },
  ],
};

const ORDER: SymbolInfo = {
  snapshot: 1,
  id: 1,
  name: "Order",
  qualified_name: "App.Domain.Order",
  kind: "class",
  visibility: "public",
  is_static: false,
  path: "src/order.cs",
  first_line: 1,
  last_line: 20,
  signature: null,
  doc: null,
  breadcrumb: [],
  metrics: [],
  members: [
    {
      id: 10,
      name: "Total",
      kind: "method",
      visibility: "public",
      signature: "int Total()",
      complexity: 2,
      complexity_rating: "good",
      usages: 3,
      first_line: 5,
      last_line: 8,
    },
    {
      id: 11,
      name: "Id",
      kind: "property",
      visibility: "public",
      signature: "int",
      complexity: null,
      complexity_rating: null,
      usages: 1,
      first_line: 3,
      last_line: 3,
    },
  ],
};

const labels = (rows: { label: string }[]) => rows.map((row) => row.label);

describe("buildRows", () => {
  it("shows only the layers while everything is closed", () => {
    const rows = buildRows(OUTLINE, new Set(), new Map(), "", LABELS);
    expect(labels(rows)).toEqual(["Domain", "none"]);
    expect(rows[0]?.meta).toBe("2");
  });

  it("opens a layer into its namespaces and a namespace into its types", () => {
    const expanded = new Set([layerKey("Domain"), namespaceKey("Domain", "App.Domain")]);
    const rows = buildRows(OUTLINE, expanded, new Map(), "", LABELS);
    expect(labels(rows)).toEqual(["Domain", "App.Domain", "Order", "IOrder", "none"]);
    expect(rows[2]?.symbolKind).toBe("class");
    expect(rows[3]?.expandable).toBe(false);
  });

  it("opens a type into groups with their members once they are read", () => {
    const expanded = new Set([
      layerKey("Domain"),
      namespaceKey("Domain", "App.Domain"),
      typeKey(1),
    ]);
    const rows = buildRows(OUTLINE, expanded, new Map([[1, ORDER]]), "", LABELS);
    expect(labels(rows)).toEqual([
      "Domain",
      "App.Domain",
      "Order",
      "Fields",
      "Methods",
      "IOrder",
      "none",
    ]);
  });

  it("a filter shows the matching types with their parents opened", () => {
    const rows = buildRows(OUTLINE, new Set(), new Map(), "ord", LABELS);
    expect(labels(rows)).toEqual(["Domain", "App.Domain", "Order", "IOrder"]);
    expect(buildRows(OUTLINE, new Set(), new Map(), "zzz", LABELS)).toEqual([]);
  });

  it("names the layer and the namespace that have no name", () => {
    const rows = buildRows(OUTLINE, new Set([layerKey("")]), new Map(), "", LABELS);
    expect(labels(rows)).toContain("root");
  });
});

describe("revealKeys", () => {
  it("names the layer, the namespace and the type", () => {
    expect(revealKeys(OUTLINE, 2)).toEqual([
      layerKey("Domain"),
      namespaceKey("Domain", "App.Domain"),
      typeKey(2),
    ]);
  });

  it("is empty for an unknown type", () => {
    expect(revealKeys(OUTLINE, 99)).toEqual([]);
  });
});
