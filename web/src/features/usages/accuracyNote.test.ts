import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import type { Level2Report, ScanReport } from "../../shared/project/report";
import { accuracyNote } from "./accuracyNote";
import type { UsagesAnswer } from "./types";

function report(languages: Level2Report["languages"]): ScanReport {
  return {
    last_scan: { at: 1, level: "2", origin: "terminal" },
    level2: { at: 1, origin: "terminal", languages },
  };
}

function answer(language: string, scan_report?: ScanReport | null): UsagesAnswer {
  return { language, scan_report } as UsagesAnswer;
}

beforeEach(() => setLanguage("en"));

describe("the accuracy note of the usages card", () => {
  it("is empty for an answer of an older version", () => {
    expect(accuracyNote(answer("rust"))).toBeNull();
  });

  it("says that no stage 2 run is stored", () => {
    expect(accuracyNote(answer("rust", null))).toMatch(/No level 2 run is stored/);
    expect(accuracyNote(answer("rust", { ...report([]), level2: null }))).toMatch(/No level 2 run/);
  });

  it("tells apart checked, checked in part, skipped, failed and no indexer", () => {
    const scan = report([
      { language: "rust", outcome: "checked", files: 12, stale: 0 },
      { language: "csharp", outcome: "checked", files: 5, stale: 2 },
      { language: "java", outcome: "skipped", reason: "busy" },
      { language: "python", outcome: "failed", code: "RES-502" },
    ]);
    expect(accuracyNote(answer("rust", scan))).toBe("Level 2 checked Rust: 12 files.");
    expect(accuracyNote(answer("csharp", scan))).toMatch(/in part: 5 files, 2 changed since/);
    expect(accuracyNote(answer("java", scan))).toMatch(/skipped Java: another stage 2 run/);
    expect(accuracyNote(answer("python", scan))).toMatch(/failed for Python \(RES-502\)/);
    expect(accuracyNote(answer("go", scan))).toMatch(/no indexer for Go/);
  });
});
