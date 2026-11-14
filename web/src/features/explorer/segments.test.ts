import { describe, expect, it } from "vitest";
import { segments } from "./segments";

const LINES = ["return calculator.Add(1, 2);", "// ünï Add"];

describe("segments", () => {
  it("gives the lines back when the pieces are put together", () => {
    const spans = [
      { line: 0, start: 7, end: 17, symbol: 5 },
      { line: 0, start: 18, end: 21, symbol: 6 },
      { line: 1, start: 7, end: 10, symbol: 6 },
    ];
    const pieces = segments(LINES, spans);
    expect(pieces.map((piece) => piece.text).join("")).toBe(LINES.join("\n"));
    expect(pieces.filter((piece) => piece.symbol !== null).map((piece) => piece.text)).toEqual([
      "calculator",
      "Add",
      "Add",
    ]);
  });

  it("is one piece per line without spans", () => {
    expect(segments(LINES, [])).toEqual([
      { text: "return calculator.Add(1, 2);\n", symbol: null },
      { text: "// ünï Add", symbol: null },
    ]);
  });

  it("drops a span that overlaps the one before it or has no width", () => {
    const pieces = segments(LINES, [
      { line: 0, start: 7, end: 17, symbol: 5 },
      { line: 0, start: 9, end: 12, symbol: 6 },
      { line: 0, start: 20, end: 20, symbol: 7 },
    ]);
    expect(pieces.filter((piece) => piece.symbol !== null)).toHaveLength(1);
    expect(pieces.map((piece) => piece.text).join("")).toBe(LINES.join("\n"));
  });
});
