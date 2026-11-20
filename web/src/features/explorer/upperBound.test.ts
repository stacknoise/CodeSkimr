import { describe, expect, it } from "vitest";
import type { ScanReport } from "../../shared/project/report";
import { isUpperBound } from "./upperBound";

function report(outcome: "checked" | "skipped" | "failed"): ScanReport {
  return {
    last_scan: { at: 1, level: "2", origin: "terminal" },
    level2: { at: 1, origin: "terminal", languages: [{ language: "rust", outcome }] },
  };
}

describe("isUpperBound", () => {
  it("is false only where level 2 checked the language", () => {
    expect(isUpperBound(report("checked"), "rust")).toBe(false);
  });

  it("is true for a skipped or failed language, another language and a missing second pass", () => {
    expect(isUpperBound(report("skipped"), "rust")).toBe(true);
    expect(isUpperBound(report("failed"), "rust")).toBe(true);
    expect(isUpperBound(report("checked"), "python")).toBe(true);
    expect(isUpperBound({ ...report("checked"), level2: null }, "rust")).toBe(true);
  });

  it("is true without a stored report", () => {
    expect(isUpperBound(null, "rust")).toBe(true);
    expect(isUpperBound(undefined, "rust")).toBe(true);
  });
});
