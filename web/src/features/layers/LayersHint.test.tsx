import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { LAYERS_ROUTE } from "../../app/consts";
import { currentPath } from "../../app/router";
import { setLanguage } from "../../shared/i18n/i18n";
import { hasLayers } from "../explorer/Overview";
import type { Outline } from "../explorer/types";
import { LayersHint } from "./LayersHint";

beforeEach(() => setLanguage("en"));

const THRESHOLDS = {
  complexity_good_max: 1,
  complexity_medium_max: 2,
  maintainability_good_min: 3,
  maintainability_medium_min: 2,
  instability_stable_max: 0.3,
  instability_medium_max: 0.6,
};

function outline(...names: string[]): Outline {
  return {
    snapshot: 1,
    thresholds: THRESHOLDS,
    layers: names.map((name) => ({ name, namespaces: [] })),
  };
}

describe("LayersHint", () => {
  it("says that no layers are set and opens the editor", () => {
    render(() => <LayersHint />);
    expect(screen.getByText("No layers set yet")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Set layers" }));
    expect(currentPath()).toBe(LAYERS_ROUTE.path);
  });
});

describe("hasLayers", () => {
  it("does not count the group of types without a layer", () => {
    expect(hasLayers(outline(""))).toBe(false);
    expect(hasLayers(outline())).toBe(false);
    expect(hasLayers(outline("domain", ""))).toBe(true);
  });
});
