import { describe, expect, it } from "vitest";
import {
  buildRows,
  groupKey,
  layerKey,
  namespaceKey,
  revealKeys,
  siblingPlaces,
  typeKey,
  type TreeLabels,
  type TreeRow,
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
            { id: 1, name: "Order", kind: "class", members: 2, language: "csharp" },
            { id: 2, name: "IOrder", kind: "interface", members: 0, language: "csharp" },
          ],
        },
      ],
    },
    {
      name: "",
      namespaces: [
        {
          name: "",
          types: [{ id: 3, name: "Loose", kind: "struct", members: 0, language: "csharp" }],
        },
      ],
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
  language: "csharp",
  first_line: 1,
  last_line: 20,
  signature: null,
  doc: null,
  layer: null,
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

/** The keys of the two segments of `App.Domain`, `App` and `App.Domain`. */
const APP = [namespaceKey("Domain", "App"), namespaceKey("Domain", "App.Domain")];

const labels = (rows: { label: string }[]) => rows.map((row) => row.label);

describe("buildRows", () => {
  it("shows only the layers while everything is closed", () => {
    const rows = buildRows(OUTLINE, new Set(), new Map(), "", LABELS);
    expect(labels(rows)).toEqual(["Domain", "none"]);
    expect(rows[0]?.meta).toBe("2");
  });

  it("opens a layer into its namespaces and a namespace into its types", () => {
    const expanded = new Set([layerKey("Domain"), ...APP]);
    const rows = buildRows(OUTLINE, expanded, new Map(), "", LABELS);
    expect(labels(rows)).toEqual(["Domain", "App", "Domain", "Order", "IOrder", "none"]);
    expect(rows[3]?.symbolKind).toBe("class");
    expect(rows[4]?.expandable).toBe(false);
  });

  it("opens a type into groups with their members once they are read", () => {
    const expanded = new Set([layerKey("Domain"), ...APP, typeKey(1)]);
    const rows = buildRows(OUTLINE, expanded, new Map([[1, ORDER]]), "", LABELS);
    expect(labels(rows)).toEqual([
      "Domain",
      "App",
      "Domain",
      "Order",
      "Fields",
      "Methods",
      "IOrder",
      "none",
    ]);
  });

  it("a filter shows the matching types with their parents opened", () => {
    const rows = buildRows(OUTLINE, new Set(), new Map(), "ord", LABELS);
    expect(labels(rows)).toEqual(["Domain", "App", "Domain", "Order", "IOrder"]);
    expect(buildRows(OUTLINE, new Set(), new Map(), "zzz", LABELS)).toEqual([]);
  });

  it("names the layer and the namespace that have no name", () => {
    const rows = buildRows(OUTLINE, new Set([layerKey("")]), new Map(), "", LABELS);
    expect(labels(rows)).toContain("root");
  });
});

describe("namespaces grouped by their parts", () => {
  const type = (id: number, name: string) => ({
    id,
    name,
    kind: "class",
    members: 0,
    language: "csharp",
  });
  const GROUPED: Outline = {
    ...OUTLINE,
    layers: [
      {
        name: "Test",
        namespaces: [
          { name: "Co.App", types: [type(1, "Root")] },
          { name: "Co.App.Catalog", types: [type(2, "Item")] },
          { name: "Co.App.Cutting", types: [type(3, "Cut")] },
          { name: "Co.Domain::Errors", types: [type(4, "Fault")] },
          { name: "", types: [type(5, "Loose")] },
        ],
      },
    ],
  };
  const all = [
    layerKey("Test"),
    ...[
      "Co",
      "Co.App",
      "Co.App.Catalog",
      "Co.App.Cutting",
      "Co.Domain",
      "Co.Domain::Errors",
      "",
    ].map((path) => namespaceKey("Test", path)),
  ];

  it("leaves out the row for no layer when the project has no layers at all", () => {
    const flat: Outline = {
      ...OUTLINE,
      layers: [{ name: "", namespaces: [{ name: "Co", types: [type(1, "Root")] }] }],
    };
    const rows = buildRows(flat, new Set([namespaceKey("", "Co")]), new Map(), "", LABELS);
    expect(rows.map((row) => [row.label, row.depth])).toEqual([
      ["Co", 0],
      ["Root", 1],
    ]);
  });

  it("shows one row for each part, with the parts below it", () => {
    const rows = buildRows(GROUPED, new Set(all), new Map(), "", LABELS);
    expect(rows.map((row) => `${row.depth}:${row.label}:${row.meta}`)).toEqual([
      "0:Test:5",
      "1:Co:4",
      "2:App:3",
      "3:Catalog:1",
      "4:Item:0",
      "3:Cutting:1",
      "4:Cut:0",
      "3:Root:0",
      "2:Domain:1",
      "3:Errors:1",
      "4:Fault:0",
      "1:root:1",
      "2:Loose:0",
    ]);
  });

  it("opens a part on its own and counts the types below it while it is closed", () => {
    const rows = buildRows(GROUPED, new Set([layerKey("Test")]), new Map(), "", LABELS);
    expect(rows.map((row) => `${row.label}:${row.meta}:${row.open}`)).toEqual([
      "Test:5:true",
      "Co:4:false",
      "root:1:false",
    ]);
  });

  it("keeps the parts that lead to a match when filtering, and drops the others", () => {
    const rows = buildRows(GROUPED, new Set(), new Map(), "cut", LABELS);
    expect(labels(rows)).toEqual(["Test", "Co", "App", "Cutting", "Cut"]);
  });

  it("keeps `App.Domain` and `App::Domain` apart, and a name of only separators is the unnamed one", () => {
    const outline: Outline = {
      ...OUTLINE,
      layers: [
        {
          name: "Test",
          namespaces: [
            { name: "App.Domain", types: [type(1, "A")] },
            { name: "App::Domain", types: [type(2, "B")] },
            { name: "::", types: [type(3, "C")] },
          ],
        },
      ],
    };
    const keys = (id: number) => revealKeys(outline, id);
    expect(keys(1)).not.toEqual(keys(2));
    expect(keys(3)).toEqual([layerKey("Test"), namespaceKey("Test", ""), typeKey(3)]);
    const open = new Set([layerKey("Test"), ...keys(1), ...keys(2), ...keys(3)]);
    const rows = buildRows(outline, open, new Map(), "", LABELS);
    expect(labels(rows)).toEqual(["Test", "App", "Domain", "A", "Domain", "B", "root", "C"]);
  });

  it("names every part that has to be open for a type, also with `::`", () => {
    expect(revealKeys(GROUPED, 4)).toEqual([
      layerKey("Test"),
      namespaceKey("Test", "Co"),
      namespaceKey("Test", "Co.Domain"),
      namespaceKey("Test", "Co.Domain::Errors"),
      typeKey(4),
    ]);
    expect(revealKeys(GROUPED, 5)).toEqual([
      layerKey("Test"),
      namespaceKey("Test", ""),
      typeKey(5),
    ]);
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
  const open = (...extra: string[]) => new Set([layerKey("Domain"), ...APP, typeKey(1), ...extra]);

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
  it("names the layer, every part of the namespace and the type", () => {
    expect(revealKeys(OUTLINE, 2)).toEqual([layerKey("Domain"), ...APP, typeKey(2)]);
  });

  it("is empty for an unknown type", () => {
    expect(revealKeys(OUTLINE, 99)).toEqual([]);
  });
});

describe("siblingPlaces", () => {
  const base: TreeRow = {
    key: "k",
    kind: "type",
    depth: 0,
    label: "",
    meta: "",
    language: null,
    symbolKind: null,
    layerIndex: null,
    expandable: false,
    open: false,
    type: null,
    member: null,
  };
  it("counts the siblings of each parent, not the rows of the list", () => {
    const at = (depth: number): TreeRow => ({ ...base, depth });
    const places = siblingPlaces([at(0), at(1), at(1), at(2), at(1), at(0)]);
    expect(places.map((place) => [place.position, place.size])).toEqual([
      [1, 2],
      [1, 3],
      [2, 3],
      [1, 1],
      [3, 3],
      [2, 2],
    ]);
  });
});
