import { describe, expect, it } from "vitest";
import {
  buildRows,
  groupKey,
  layerKey,
  namespaceKey,
  revealKeys,
  typeKey,
  type TreeLabels,
} from "./tree";
import type { Outline, SymbolInfo } from "./types";

const LABELS: TreeLabels = {
  noLayer: "none",
  noNamespace: "root",
  fields: "Fields",
  methods: "Methods",
  nested: "Nested",
};

const OUTLINE: Outline = {
  snapshot: 1,
  thresholds: {
    complexity_good_max: 5,
    complexity_medium_max: 10,
    maintainability_good_min: 65,
    maintainability_medium_min: 35,
    instability_stable_max: 0.3,
    instability_medium_max: 0.7,
  },
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

describe("nested types", () => {
  const NESTED = {
    ...ORDER,
    members: [
      ...ORDER.members,
      { ...ORDER.members[0]!, id: 12, name: "Line", kind: "class", signature: null },
    ],
  };
  const open = (...extra: string[]) =>
    new Set([layerKey("Domain"), namespaceKey("Domain", "App.Domain"), typeKey(1), ...extra]);

  it("gives them a group of their own, closed at first", () => {
    const rows = buildRows(OUTLINE, open(), new Map([[1, NESTED]]), "", LABELS);
    expect(labels(rows)).toContain("Nested");
    expect(labels(rows)).not.toContain("Line");
  });

  it("shows them as type rows below the group once it is open", () => {
    const group = groupKey(1, "nested");
    const rows = buildRows(OUTLINE, open(group), new Map([[1, NESTED]]), "", LABELS);
    const line = rows.find((row) => row.label === "Line");
    expect(line?.kind).toBe("type");
    expect(line?.type).toBe(12);
    expect(line?.depth).toBeGreaterThan(rows.find((row) => row.label === "Nested")?.depth ?? 0);
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
