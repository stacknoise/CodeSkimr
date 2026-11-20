import { beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";

const config = vi.hoisted(() => ({ value: null as { approximate_languages?: string[] } | null }));
vi.mock("../../shared/settings/ui-config", () => ({ uiConfig: () => config.value }));

import { approximationNote, isApproximate } from "./approximation";

beforeEach(() => {
  setLanguage("en");
  config.value = { approximate_languages: ["python", "csharp"] };
});

describe("the approximation of a language", () => {
  it("marks the languages the server lists", () => {
    expect(isApproximate("python")).toBe(true);
    expect(isApproximate("rust")).toBe(false);
    expect(isApproximate(undefined)).toBe(false);
  });

  it("marks nothing when the server tells nothing", () => {
    config.value = null;
    expect(isApproximate("python")).toBe(false);
    config.value = {};
    expect(isApproximate("python")).toBe(false);
  });

  it("explains the approximation with the name of the language", () => {
    expect(approximationNote("csharp")).toMatch(/^C# is an approximation/);
    expect(approximationNote("rust")).toBeNull();
  });
});
