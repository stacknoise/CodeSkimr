import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { tilesOf, tipFor } from "./metrics";
import type { SymbolInfo, Thresholds } from "./types";

const LIMITS: Thresholds = {
  complexity_good_max: 5,
  complexity_medium_max: 10,
  maintainability_good_min: 65,
  maintainability_medium_min: 35,
  instability_stable_max: 0.3,
  instability_medium_max: 0.7,
};

const INFO = {
  members: [
    { kind: "method", id: 1 },
    { kind: "method", id: 2 },
    { kind: "property", id: 3 },
  ],
  metrics: [
    { kind: "cc", value: 12, rating: "poor" },
    { kind: "mi", value: 70, rating: "good" },
  ],
} as unknown as SymbolInfo;

beforeEach(() => setLanguage("en"));

describe("tilesOf", () => {
  it("makes ten tiles", () => {
    expect(tilesOf(INFO, LIMITS, "en")).toHaveLength(10);
  });

  it("gives a stored metric its number and its rating together", () => {
    const cc = tilesOf(INFO, LIMITS, "en").find((tile) => tile.id === "cc");
    expect(cc?.value).toBe("12");
    expect(cc?.rating).toBe("poor");
  });

  it("counts the members by kind and shows a dash for a metric that was not measured", () => {
    const tiles = tilesOf(INFO, LIMITS, "en");
    expect(tiles.find((tile) => tile.id === "methods")?.value).toBe("2");
    expect(tiles.find((tile) => tile.id === "instability")?.value).toBe("–");
  });

  it("names the limits in the tips of the metrics that have some", () => {
    const tiles = tilesOf(INFO, LIMITS, "en");
    expect(tiles.find((tile) => tile.id === "cc")?.tip).toMatch(/5.*10/);
    expect(tiles.find((tile) => tile.id === "mi")?.tip).toMatch(/65.*35/);
  });
});

describe("tipFor", () => {
  it("names the limits only when the thresholds are known", () => {
    expect(tipFor("cc", I18N_KEYS.EXPLORER_METRIC_CC_TIP, LIMITS)).toMatch(/5.*10/);
    expect(tipFor("cc", I18N_KEYS.EXPLORER_METRIC_CC_TIP, null)).not.toMatch(/\d/);
  });
});
