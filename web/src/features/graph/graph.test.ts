import { describe, expect, it } from "vitest";
import { edgeKind, edgePath, shown } from "./geometry";
import { neighboursOf, violations } from "./neighbours";
import type { LayerBox, LayerLink } from "./types";
import { fit, pan, zoomAt } from "./viewport";

const box = (name: string, x: number, y: number): LayerBox => ({
  name,
  x,
  y,
  width: 100,
  height: 40,
  rank: 0,
  order: 0,
  layer: null,
  types: null,
});
const link = (from: string, to: string, count = 1, violates = false): LayerLink => ({
  from,
  to,
  count,
  violates_layering: violates,
  in_cycle: false,
});

describe("edges", () => {
  it("colours the edges of the selection by direction and always marks a break red", () => {
    expect(edgeKind(link("ui", "core"), "ui")).toBe("out");
    expect(edgeKind(link("ui", "core"), "core")).toBe("in");
    expect(edgeKind(link("ui", "core"), "db")).toBe("weak");
    expect(edgeKind(link("core", "ui", 1, true), "ui")).toBe("violation");
  });

  it("hides every edge but the breaks with 'violations only', and edges to oneself always", () => {
    expect(shown(link("a", "b"), true)).toBe(false);
    expect(shown(link("a", "b", 1, true), true)).toBe(true);
    expect(shown(link("a", "a"), false)).toBe(false);
  });

  it("leaves the bottom of the upper box and enters the top of the lower one", () => {
    expect(edgePath(box("a", 0, 0), box("b", 200, 100))).toBe("M50 40 C50 70 250 70 250 100");
    expect(edgePath(box("b", 200, 100), box("a", 0, 0))).toBe("M250 100 C250 70 50 70 50 40");
  });

  it("loops above two boxes of one row", () => {
    expect(edgePath(box("a", 0, 100), box("b", 200, 100))).toBe("M50 100 C50 66 250 66 250 100");
  });
});

describe("neighbours", () => {
  const links = [
    link("ui", "core", 5),
    link("ui", "db", 2),
    link("db", "core", 3),
    link("ui", "ui", 9),
  ];

  it("lists what a layer uses and what uses it, and sums the references", () => {
    const result = neighboursOf("ui", links);
    expect(result.uses.map((item) => item.to)).toEqual(["core", "db"]);
    expect(result.usedBy).toEqual([]);
    expect(result.references).toBe(7);
    expect(neighboursOf("core", links).usedBy.map((item) => item.from)).toEqual(["ui", "db"]);
  });

  it("lists the breaks of the layering, most references first", () => {
    const broken = violations([link("a", "b", 1, true), link("c", "d", 4, true), link("e", "f")]);
    expect(broken.map((item) => item.from)).toEqual(["c", "a"]);
  });
});

describe("viewport", () => {
  it("fits the picture into the area, centred, and never enlarges it", () => {
    expect(fit({ width: 1000, height: 500 }, { width: 564, height: 400 })).toEqual({
      scale: 0.5,
      x: 32,
      y: 75,
    });
    expect(fit({ width: 10, height: 10 }, { width: 500, height: 500 }).scale).toBe(1);
    expect(fit({ width: 0, height: 0 }, { width: 500, height: 500 })).toEqual({
      scale: 1,
      x: 0,
      y: 0,
    });
  });

  it("zooms around the pointer so that the point under it stays", () => {
    const view = { scale: 1, x: 0, y: 0 };
    const zoomed = zoomAt(view, -5, { x: 100, y: 50 });
    expect(zoomed.scale).toBeGreaterThan(1);
    expect(100 * (zoomed.scale - 1) + zoomed.x).toBeCloseTo(0);
    expect(zoomAt(view, 1000, { x: 0, y: 0 }).scale).toBe(0.2);
    expect(zoomAt(view, -1000, { x: 0, y: 0 }).scale).toBe(3);
  });

  it("pans by the distance of the pointer", () => {
    expect(pan({ scale: 2, x: 1, y: 2 }, 10, -5)).toEqual({ scale: 2, x: 11, y: -3 });
  });
});
