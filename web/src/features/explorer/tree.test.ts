import { describe, expect, it } from "vitest";
import {
  blankRow,
  buildRows,
  groupKey,
  groupOfKind,
  layerKey,
  namespaceKey,
  revealKeys,
  revealNamespace,
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
  aliases: "Aliases",
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
            {
              id: 1,
              name: "Order",
              qualified_name: "Order",
              kind: "class",
              members: 2,
              language: "csharp",
            },
            {
              id: 2,
              name: "IOrder",
              qualified_name: "IOrder",
              kind: "interface",
              members: 0,
              language: "csharp",
            },
          ],
        },
      ],
    },
    {
      name: "",
      namespaces: [
        {
          name: "",
          types: [
            {
              id: 3,
              name: "Loose",
              qualified_name: "Loose",
              kind: "struct",
              members: 0,
              language: "csharp",
            },
          ],
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
      qualified_name: "Total",
      kind: "method",
      visibility: "public",
      signature: "int Total()",
      parameters: [],
      complexity: 2,
      complexity_rating: "good",
      usages: 3,
      first_line: 5,
      last_line: 8,
    },
    {
      id: 11,
      name: "Id",
      qualified_name: "Id",
      kind: "property",
      visibility: "public",
      signature: "int",
      parameters: [],
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

describe("the annotation targets of the rows", () => {
  const expanded = new Set([
    layerKey("Domain"),
    ...APP,
    typeKey(1),
    groupKey(1, "methods"),
    groupKey(1, "fields"),
  ]);

  it("name the symbol of a namespace, a type and a member by kind and qualified name", () => {
    const rows = buildRows(OUTLINE, expanded, new Map([[1, ORDER]]), "", LABELS);
    const namespace = rows.find((row) => row.label === "Domain" && row.kind === "namespace");
    const type = rows.find((row) => row.kind === "type");
    const member = rows.find((row) => row.kind === "member" && row.label === "Total");
    expect(namespace?.target).toEqual({ kind: "namespace", name: "App.Domain" });
    expect(type?.target).toEqual({ kind: "class", name: "Order" });
    expect(member?.target).toEqual({ kind: "method", name: "Total" });
  });

  it("give a folded namespace and type the name whose critical symbols below are counted", () => {
    const rows = buildRows(OUTLINE, expanded, new Map([[1, ORDER]]), "", LABELS);
    expect(rows.find((row) => row.kind === "type")?.scope).toBe("Order");
    expect(rows.find((row) => row.kind === "namespace")?.scope).toBe("App");
  });

  it("give a group the qualified names of its members, and a layer no symbol", () => {
    const rows = buildRows(OUTLINE, expanded, new Map([[1, ORDER]]), "", LABELS);
    expect(rows.find((row) => row.kind === "group")?.names.length).toBeGreaterThan(0);
    expect(rows.find((row) => row.kind === "layer")?.target).toBeNull();
  });
});

describe("the description of a package", () => {
  it("is shown after the name of the namespace row that has one", () => {
    const first = OUTLINE.layers[0]?.namespaces[0];
    if (!first) {
      throw new Error("the sample outline has no namespace");
    }
    const described: Outline = {
      ...OUTLINE,
      layers: [
        { name: "", namespaces: [{ ...first, name: "zcore", description: "Order & Billing" }] },
      ],
    };
    const rows = buildRows(described, new Set(), new Map(), "", LABELS);
    expect(rows.find((row) => row.label === "zcore")?.description).toBe("Order & Billing");
    const plain = buildRows(OUTLINE, new Set(), new Map(), "", LABELS);
    expect(plain.every((row) => row.description === undefined)).toBe(true);
  });
});

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

/** A symbol row of the outline: `qualified` defaults to the name. */
const symbolOf =
  (kind: string, language: string) =>
  (id: number, name: string, qualified = name) => ({
    id,
    name,
    qualified_name: qualified,
    kind,
    members: 0,
    language,
  });

describe("namespaces grouped by their parts", () => {
  const type = symbolOf("class", "csharp");
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
  const base: TreeRow = blankRow("k", "type", 0, "");
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

describe("revealNamespace", () => {
  const outline = {
    snapshot: 1,
    thresholds: {} as never,
    layers: [
      {
        name: "Domain",
        namespaces: [
          { name: "App.Domain.Model", types: [] },
          { name: "App.Ui", types: [] },
        ],
      },
    ],
  };

  it("opens the layer and the segments down to the namespace and puts the cursor on it", () => {
    const found = revealNamespace(outline, "App.Domain");
    expect(found?.cursor).toBe(namespaceKey("Domain", "App.Domain"));
    expect(found?.open).toEqual([
      layerKey("Domain"),
      namespaceKey("Domain", "App"),
      namespaceKey("Domain", "App.Domain"),
    ]);
  });

  it("finds nothing for a name that is no part of a namespace", () => {
    expect(revealNamespace(outline, "Shop")).toBeNull();
  });
});

describe("modules in the tree", () => {
  const python = symbolOf("module", "python");
  const module = (id: number, name: string) => python(id, name, `pkg.${name}`);
  const PYTHON: Outline = {
    ...OUTLINE,
    layers: [
      {
        name: "",
        namespaces: [
          { name: "pkg.__init__", types: [], module: module(10, "__init__") },
          {
            name: "pkg.entry",
            module: module(11, "entry"),
            types: [
              {
                id: 12,
                name: "Entry",
                qualified_name: "pkg.entry.Entry",
                kind: "class",
                members: 0,
                language: "python",
              },
            ],
          },
        ],
      },
    ],
  };
  const open = new Set([namespaceKey("", "pkg")]);

  it("shows a module without types as a row that opens the module", () => {
    const rows = buildRows(PYTHON, open, new Map(), "", LABELS);
    const init = rows.find((row) => row.label === "__init__");
    expect(init?.symbolKind).toBe("module");
    expect(init?.type).toBe(10);
    expect(init?.expandable).toBe(false);
  });

  it("finds a module by its name when filtering", () => {
    const rows = buildRows(PYTHON, new Set(), new Map(), "init", LABELS);
    expect(rows.map((row) => row.label)).toEqual(["pkg", "__init__"]);
  });

  it("reveals a module at its namespace row", () => {
    expect(revealKeys(PYTHON, 11).at(-1)).toBe(namespaceKey("", "pkg.entry"));
  });
});

describe("members of a module and of every kind", () => {
  const member = (id: number, name: string, kind: string) => ({
    ...ORDER.members[0]!,
    id,
    name,
    qualified_name: name,
    kind,
  });
  const MODULE: SymbolInfo = {
    ...ORDER,
    id: 11,
    name: "sensor",
    kind: "module",
    language: "python",
    members: [
      member(20, "setup", "function"),
      member(21, "GROUPS", "constant"),
      member(22, "Sensor", "class"),
      member(23, "Alias", "type_alias"),
    ],
  };
  const outline: Outline = {
    ...OUTLINE,
    layers: [
      {
        name: "",
        namespaces: [
          {
            name: "pkg.sensor",
            types: [
              {
                id: 22,
                name: "Sensor",
                qualified_name: "pkg.sensor.Sensor",
                kind: "class",
                members: 0,
                language: "python",
              },
            ],
            module: {
              id: 11,
              name: "sensor",
              qualified_name: "pkg.sensor",
              kind: "module",
              members: 4,
              language: "python",
            },
          },
        ],
      },
    ],
  };
  const open = new Set([namespaceKey("", "pkg"), namespaceKey("", "pkg.sensor")]);
  const groups = (rows: TreeRow[]) =>
    rows.filter((row) => row.kind === "group").map((row) => row.label);

  it("shows the constants, functions and aliases of a module next to its types", () => {
    const rows = buildRows(outline, open, new Map([[11, MODULE]]), "", LABELS);
    expect(groups(rows)).toEqual(["Fields", "Methods", "Aliases"]);
    expect(labels(rows)).toContain("Sensor");
  });

  it("keeps the groups of a module closed until its row is open and read", () => {
    const closed = new Set([namespaceKey("", "pkg")]);
    expect(groups(buildRows(outline, closed, new Map([[11, MODULE]]), "", LABELS))).toEqual([]);
    expect(groups(buildRows(outline, open, new Map(), "", LABELS))).toEqual([]);
  });

  it("puts every member kind into one group, or into the types of the namespace", () => {
    const placed = (kind: string) => groupOfKind(kind);
    expect(["property", "field", "constant", "event", "enum_member"].map(placed)).toEqual(
      Array(5).fill("fields"),
    );
    expect(["method", "constructor", "function"].map(placed)).toEqual(Array(3).fill("methods"));
    expect(placed("type_alias")).toBe("aliases");
    expect(
      ["class", "interface", "trait", "record", "struct", "enum", "union"].map(placed),
    ).toEqual(Array(7).fill("nested"));
  });
});

describe("holders of a namespace", () => {
  const holder = (id: number, members: number) => ({
    id,
    name: "util",
    qualified_name: "app.util",
    kind: "namespace",
    members,
    language: "typescript",
  });
  const fn = (id: number, name: string) => ({ ...ORDER.members[0]!, id, name, kind: "function" });
  const info = (id: number, ...members: ReturnType<typeof fn>[]): SymbolInfo => ({
    ...ORDER,
    id,
    kind: "namespace",
    members,
  });
  const outline: Outline = {
    ...OUTLINE,
    layers: [
      {
        name: "",
        namespaces: [
          { name: "app.util", types: [], module: null, holders: [holder(5, 1), holder(6, 1)] },
        ],
      },
    ],
  };
  const open = new Set([namespaceKey("", "app"), namespaceKey("", "app.util")]);

  it("shows a namespace with functions but no types, and reads its holders when it opens", () => {
    const closed = buildRows(outline, new Set([namespaceKey("", "app")]), new Map(), "", LABELS);
    const row = closed.find((one) => one.label === "util");
    expect(row?.expandable).toBe(true);
    expect(row?.held).toEqual([5, 6]);
    expect(row?.type).toBeNull();
  });

  it("joins the functions of every holder into one group", () => {
    const details = new Map([
      [5, info(5, fn(30, "a"))],
      [6, info(6, fn(31, "b"))],
    ]);
    const opened = new Set([...open, groupKey(5, "methods")]);
    const rows = buildRows(outline, opened, details, "", LABELS);
    const group = rows.find((one) => one.kind === "group");
    expect(group?.label).toBe("Methods");
    expect(group?.meta).toBe("2");
    const members = rows.filter((one) => one.kind === "member");
    expect(members.map((one) => one.type)).toEqual([5, 6]);
  });
});

describe("files without a namespace", () => {
  const outline: Outline = {
    ...OUTLINE,
    layers: [
      {
        name: "",
        namespaces: [
          {
            name: "app/Plain.cs",
            file: true,
            types: [
              {
                id: 1,
                name: "Plain",
                qualified_name: "Plain",
                kind: "class",
                members: 0,
                language: "csharp",
              },
            ],
          },
        ],
      },
    ],
  };

  it("shows a file as one row titled by the file, not as a namespace app/Plain and cs", () => {
    const rows = buildRows(
      outline,
      new Set([namespaceKey("", "app/Plain.cs")]),
      new Map(),
      "",
      LABELS,
    );
    expect(labels(rows)).toEqual(["Plain.cs", "Plain"]);
  });

  it("reveals a type of such a file at its row", () => {
    expect(revealKeys(outline, 1)).toEqual([
      layerKey(""),
      namespaceKey("", "app/Plain.cs"),
      typeKey(1),
    ]);
  });
});
