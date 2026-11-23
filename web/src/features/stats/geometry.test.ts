import { describe, expect, it } from "vitest";
import { HISTOGRAM, HOTSPOTS, USAGE_BAR } from "./consts";
import {
  classBars,
  complexityTicks,
  histogramShapes,
  mostUsages,
  shortPath,
  spots,
  spotScale,
  usageTicks,
  usageWidths,
} from "./geometry";
import type { Row } from "./types";

function row(patch: Partial<Row>): Row {
  return {
    id: 1,
    name: "run",
    qualified_name: "app::run",
    kind: "function",
    path: "crates/app/src/run.rs",
    language: "rust",
    parent: null,
    visibility: "public",
    test: false,
    certain: 0,
    total: 0,
    complexity: null,
    complexity_rating: null,
    loc: null,
    efferent: null,
    afferent: null,
    assessment: null,
    ...patch,
  };
}

describe("shortPath", () => {
  it("keeps the last two parts", () => {
    expect(shortPath("crates/app/src/run.rs")).toBe("src/run.rs");
    expect(shortPath("run.rs")).toBe("run.rs");
  });
});

describe("usage bars", () => {
  it("gives the row with the most usages the full width", () => {
    const rows = [row({ certain: 2, total: 8 }), row({ certain: 1, total: 4 })];
    const most = mostUsages(rows);
    const widths = usageWidths(rows[0] as Row, most);
    expect(most).toBe(8);
    expect(widths.certain + widths.probable).toBeCloseTo(USAGE_BAR.width);
    expect(widths.certain).toBeCloseTo(USAGE_BAR.width / 4);
  });

  it("draws nothing when there are no usages at all", () => {
    expect(usageWidths(row({}), 0)).toEqual({ certain: 0, probable: 0 });
  });
});

describe("histogramShapes", () => {
  it("keeps a single function visible next to thousands", () => {
    const shapes = histogramShapes(
      classBars(
        [
          { complexity: 1, count: 3000 },
          { complexity: 17, count: 1 },
        ],
        10,
      ).filter((bar) => bar.count > 0),
    );
    const [many, one] = shapes;
    expect(shapes).toHaveLength(2);
    expect(one?.height).toBeGreaterThanOrEqual(HISTOGRAM.minBar);
    expect(many?.height).toBeGreaterThan(one?.height ?? 0);
    expect(many?.y).toBeCloseTo(HISTOGRAM.top);
    expect(one?.middle).toBeGreaterThan(many?.middle ?? 0);
  });

  it("draws nothing without bars", () => {
    expect(histogramShapes([])).toEqual([]);
  });
});

describe("classBars", () => {
  it("groups hundreds of complexity values into a handful of classes", () => {
    const many = Array.from({ length: 500 }, (_, index) => ({ complexity: index + 1, count: 2 }));
    const classes = classBars(many, 10);
    expect(classes.map((bar) => bar.label)).toEqual([
      "1",
      "2–4",
      "5–9",
      "10",
      "11–19",
      "20–49",
      "50–99",
      "100+",
    ]);
    expect(classes.reduce((sum, bar) => sum + bar.count, 0)).toBe(1000);
    expect(classes.filter((bar) => bar.over).map((bar) => bar.label)).toEqual([
      "11–19",
      "20–49",
      "50–99",
      "100+",
    ]);
  });

  it("counts a value below 1 in the first class", () => {
    expect(classBars([{ complexity: 0, count: 4 }], 10)[0]?.count).toBe(4);
  });

  it("starts a class at the limit of the project", () => {
    expect(classBars([], 25).find((bar) => bar.from === 26)?.over).toBe(true);
  });
});

describe("hotspots", () => {
  const rows = [
    row({ id: 1, complexity: 20, certain: 5, total: 500 }),
    row({ id: 2, complexity: 2, certain: 0, total: 0 }),
    row({ id: 3, complexity: null, certain: 9, total: 9 }),
  ];

  it("leaves out rows without a complexity", () => {
    const placed = spots(rows, spotScale(rows));
    expect(placed.map((spot) => spot.row.id)).toEqual([1, 2]);
  });

  it("puts the complex function to the right and the upper bound above the certain usages", () => {
    const [complex, simple] = spots(rows, spotScale(rows));
    expect(complex?.x).toBeCloseTo(HOTSPOTS.width - HOTSPOTS.right);
    expect(complex?.x).toBeGreaterThan(simple?.x ?? 0);
    expect(complex?.top).toBeLessThan(complex?.y ?? 0);
    expect(simple?.y).toBeCloseTo(HOTSPOTS.height - HOTSPOTS.bottom);
  });

  it("keeps one huge function from pushing the others to the edge", () => {
    const huge = [
      row({ id: 1, complexity: 3001, certain: 0, total: 1 }),
      row({ id: 2, complexity: 30, certain: 0, total: 1 }),
    ];
    const [, middle] = spots(huge, spotScale(huge));
    const share =
      ((middle?.x ?? 0) - HOTSPOTS.left) / (HOTSPOTS.width - HOTSPOTS.left - HOTSPOTS.right);
    expect(share).toBeGreaterThan(0.4);
  });

  it("keeps a least size of the scales for few rows", () => {
    const scale = spotScale([]);
    expect(scale.maxX).toBe(HOTSPOTS.minMaxX);
    expect(scale.maxY).toBe(HOTSPOTS.minMaxY);
  });

  it("steps both axes by powers of ten", () => {
    expect(usageTicks(500)).toEqual([0, 1, 10, 100]);
    expect(complexityTicks(3001)).toEqual([1, 10, 100, 1000]);
  });
});
