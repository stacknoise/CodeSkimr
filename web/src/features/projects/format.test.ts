import { describe, expect, it } from "vitest";
import { ago, count, languageName, languageShares, seconds } from "./format";

const NOW = 1_800_000_000_000;

describe("languageShares", () => {
  it("adds up to 100 and gives the rounding error to the largest language", () => {
    const shares = languageShares([
      { language: "csharp", files: 3, loc: 1 },
      { language: "typescript", files: 3, loc: 1 },
      { language: "python", files: 3, loc: 1 },
    ]);
    expect(shares.map((item) => item.percent)).toEqual([34, 33, 33]);
    expect(shares.map((item) => item.name)).toEqual(["C#", "TypeScript", "Python"]);
  });

  it("leaves out a language with less than half a percent and has no shares without code", () => {
    const shares = languageShares([
      { language: "csharp", files: 9, loc: 9_999 },
      { language: "xaml", files: 1, loc: 1 },
    ]);
    expect(shares).toHaveLength(1);
    expect(languageShares([])).toEqual([]);
  });

  it("gives a language without a colour the last one", () => {
    expect(languageShares([{ language: "xaml", files: 1, loc: 5 }])[0]?.tone).toBe("tone6");
    expect(languageName("xaml")).toBe("xaml");
  });
});

describe("ago", () => {
  it("counts in the largest fitting unit, in the language", () => {
    const seconds = NOW / 1000;
    expect(ago(seconds - 180, NOW, "en")).toBe("3 minutes ago");
    expect(ago(seconds - 180, NOW, "de")).toBe("vor 3 Minuten");
    expect(ago(seconds - 86_400, NOW, "en")).toBe("yesterday");
    expect(ago(seconds - 2 * 86_400, NOW, "de")).toBe("vorgestern");
    expect(ago(seconds + 5, NOW, "en")).toBe("now");
  });
});

describe("numbers", () => {
  it("use the notation of the language", () => {
    expect(count(41_208, "de")).toBe("41.208");
    expect(count(41_208, "en")).toBe("41,208");
    expect(seconds(1.43, "de")).toBe("1,4");
  });
});
