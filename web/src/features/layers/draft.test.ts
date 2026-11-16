import { describe, expect, it } from "vitest";
import {
  addLayer,
  addToLayer,
  covers,
  isValid,
  moveLayer,
  ownerOf,
  removeLayer,
  removeNamespace,
  renameLayer,
  sameLayers,
} from "./draft";

const LAYERS = [
  { name: "app", namespaces: ["crates::app"] },
  { name: "domain", namespaces: ["crates::domain"] },
];

describe("covers and ownerOf", () => {
  it("lets a prefix own itself and what lies below it up to a separator", () => {
    expect(covers("crates::app", "crates::app")).toBe(true);
    expect(covers("crates::app", "crates::app::ui")).toBe(true);
    expect(covers("App.Domain", "App.Domain.Order")).toBe(true);
    expect(covers("App.Domain", "App.DomainX")).toBe(false);
  });

  it("names the first layer that owns a name", () => {
    expect(ownerOf(LAYERS, "crates::domain::model")).toBe("domain");
    expect(ownerOf(LAYERS, "crates::cli")).toBeNull();
    const both = [{ name: "all", namespaces: ["crates"] }, ...LAYERS];
    expect(ownerOf(both, "crates::app")).toBe("all");
  });
});

describe("changing the layers", () => {
  it("makes a layer from a choice, trims its name and takes the prefix from others", () => {
    const next = addLayer(LAYERS, "  cli ", ["crates::cli", "crates::app"]);
    expect(next[2]).toEqual({ name: "cli", namespaces: ["crates::cli", "crates::app"] });
    expect(next[0]?.namespaces).toEqual([]);
  });

  it("does not add a prefix twice and drops what a wider prefix covers", () => {
    const wide = addToLayer(LAYERS, 1, ["crates"]);
    expect(wide[1]?.namespaces).toEqual(["crates"]);
    const again = addToLayer(wide, 1, ["crates::domain"]);
    expect(again[1]?.namespaces).toEqual(["crates"]);
  });

  it("renames, moves and removes", () => {
    expect(renameLayer(LAYERS, 0, "ui")[0]?.name).toBe("ui");
    expect(moveLayer(LAYERS, 0, 1).map((layer) => layer.name)).toEqual(["domain", "app"]);
    expect(moveLayer(LAYERS, 0, -1)).toEqual(LAYERS);
    expect(moveLayer(LAYERS, 1, 1)).toEqual(LAYERS);
    expect(removeLayer(LAYERS, 0).map((layer) => layer.name)).toEqual(["domain"]);
    expect(removeNamespace(LAYERS, 0, "crates::app")[0]?.namespaces).toEqual([]);
  });
});

describe("isValid and sameLayers", () => {
  it("wants a unique, filled name and a namespace for every layer", () => {
    expect(isValid(LAYERS)).toBe(true);
    expect(isValid([{ name: " ", namespaces: ["a"] }])).toBe(false);
    expect(isValid([{ name: "a", namespaces: [] }])).toBe(false);
    expect(isValid([LAYERS[0]!, { ...LAYERS[1]!, name: "app" }])).toBe(false);
  });

  it("compares by content and order", () => {
    expect(sameLayers(LAYERS, [...LAYERS])).toBe(true);
    expect(sameLayers(LAYERS, [...LAYERS].reverse())).toBe(false);
  });
});
