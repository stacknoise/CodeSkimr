import { render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Code } from "./Code";
import { resetHighlightCache } from "./highlight";
import { linesOf } from "./parts";

const run = vi.fn();
vi.mock("../api/commands", () => ({ runCommand: (...args: unknown[]) => run(...args) }));

describe("linesOf", () => {
  it("cuts a line at the runs and gives the text back", () => {
    const text = "fn  run(a: u32)";
    const parts = linesOf(text, [
      { line: 0, start: 0, end: 2, class: "keyword" },
      { line: 0, start: 8, end: 11, class: "type" },
    ])[0];
    expect(parts?.map((part) => part.text).join("")).toBe(text);
    expect(parts?.[0]).toEqual({ text: "fn", syntax: "keyword" });
    expect(parts?.[1]?.syntax).toBeNull();
  });

  it("drops an overlapping run and one that does not fit", () => {
    const parts = linesOf("abc", [
      { line: 0, start: 0, end: 2, class: "keyword" },
      { line: 0, start: 1, end: 3, class: "type" },
      { line: 0, start: 9, end: 12, class: "string" },
    ])[0];
    expect(parts?.map((part) => part.syntax)).toEqual(["keyword", null]);
  });

  it("keeps indentation and line breaks", () => {
    expect(linesOf("  a\n\tb", []).map((line) => line[0]?.text)).toEqual(["  a", "\tb"]);
  });
});

describe("Code", () => {
  beforeEach(() => {
    run.mockReset();
    resetHighlightCache();
  });

  it("shows the text plain first and colours it when the server answers", async () => {
    run.mockResolvedValue({
      pieces: [{ tokens: [{ line: 0, start: 0, end: 2, class: "keyword" }] }],
    });
    render(() => <Code path="a.rs" text="fn run()" />);
    expect(screen.getByText("fn run()")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("fn").tagName).toBe("SPAN"));
    expect(run).toHaveBeenCalledTimes(1);
  });

  it("asks once for the same piece and stays plain when the call fails", async () => {
    run.mockRejectedValue(new Error("no server"));
    render(() => (
      <>
        <Code path="a.rs" text="fn run()" />
        <Code path="b.rs" text="fn run()" />
      </>
    ));
    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
    expect(screen.getAllByText("fn run()")).toHaveLength(2);
  });
});
