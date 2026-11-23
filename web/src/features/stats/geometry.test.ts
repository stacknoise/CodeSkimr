import { describe, expect, it } from "vitest";
import { HISTOGRAM, HOTSPOTS, USAGE_BAR } from "./consts";
import {
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
    const shapes = histogramShapes([
      { complexity: 1, count: 3000 },
      { complexity: 17, count: 1 },
    ]);
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

  it("keeps a least size of the scales for few rows", () => {
    const scale = spotScale([]);
    expect(scale.maxX).toBe(HOTSPOTS.minMaxX);
    expect(scale.maxY).toBe(HOTSPOTS.minMaxY);
  });

  it("steps the usages by powers of ten and the complexity evenly", () => {
    expect(usageTicks(500)).toEqual([0, 1, 10, 100]);
    const ticks = complexityTicks(20);
    expect(ticks[0]).toBe(0);
    expect(ticks.at(-1)).toBeLessThanOrEqual(20);
    expect(ticks.length).toBeLessThanOrEqual(HOTSPOTS.ticksX + 1);
  });
});
