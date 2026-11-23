import { describe, expect, it } from "vitest";
import { perFullScanUnit } from "./rate";

describe("perFullScanUnit", () => {
  it("scales the time of a large scan down to 100k lines", () => {
    expect(perFullScanUnit(488_932, 4_600_000)).toBeCloseTo(10_629, 0);
  });

  it("keeps the measured time of a project below 100k lines", () => {
    expect(perFullScanUnit(800, 20_000)).toBe(800);
  });

  it("keeps the plain time of a project without a line count", () => {
    expect(perFullScanUnit(500, 0)).toBe(500);
  });
});
