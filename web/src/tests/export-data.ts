import type { Hit, UsagesAnswer } from "../features/usages/types";
import type { ExportData } from "../export/types";

function hit(depth: number, kind: string): Hit {
  const place = { id: 1, qualified_name: "a::b", kind: "method", layer: null };
  return {
    depth,
    kind,
    confidence: "certain",
    verified: false,
    path: "a.rs",
    line: 1,
    from: place,
    to: place,
  };
}

export const SUMMARY = { places: 3, files: 1, layers: 0, verified: 0, unchecked: 3, probable: 0 };

const FOCUS: UsagesAnswer = {
  snapshot: 4,
  symbol: { id: 1, qualified_name: "a::Foo", kind: "struct", layer: null },
  signature: null,
  path: "a.rs",
  depth: 3,
  incoming: { total: 3, hits: [hit(1, "call"), hit(1, "test"), hit(2, "call")] },
  outgoing: { total: 1, hits: [hit(1, "read")] },
  summary: SUMMARY,
  impact: { total: 2, public_interfaces: 0, verified: 0, unchecked: 2, layers: [] },
  level2: "unavailable",
};

function symbol(id: number, name: string, qualified: string, parent: number | null) {
  return {
    snapshot: 4,
    id,
    name,
    qualified_name: qualified,
    kind: "method",
    visibility: null,
    is_static: false,
    path: "a.rs",
    first_line: 3,
    last_line: 9,
    signature: null,
    doc: null,
    breadcrumb: parent === null ? [] : [{ id: parent, name: "Owner", kind: "struct" }],
    metrics: [],
    members: [],
  };
}

export const FOCUS_ANSWER = FOCUS;
export const DATA = {
  format: 1,
  snapshot: 4,
  project: {
    id: 9,
    name: "demo",
    commit: null,
    created: 100,
    mode: "full",
    files: 1,
    symbols: 3,
    references: 5,
  },
  outline: {
    snapshot: 4,
    layers: [
      {
        name: "",
        namespaces: [{ name: "a", types: [{ id: 1, name: "Foo", kind: "struct", members: 1 }] }],
      },
    ],
    thresholds: {},
  },
  symbols: {
    1: symbol(1, "Foo", "a::Foo", null),
    2: symbol(2, "frobnicate", "a::Foo::frobnicate", 1),
    3: symbol(3, "bar", "a::bar", null),
  },
  usages: { 1: { answer: FOCUS, summary_without_tests: { ...SUMMARY, places: 1, unchecked: 1 } } },
  graph: { snapshot: 4, width: 0, height: 0, boxes: [], links: [] },
} as unknown as ExportData;
