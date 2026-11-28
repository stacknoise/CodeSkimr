import { render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { For } from "solid-js";
import { Code } from "./Code";
import { CACHE_LIMIT } from "./consts";
import { cacheSize, resetHighlightCache } from "./highlight";
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

describe("linesOf with a mark", () => {
  it("cuts the parts at the borders of the mark and keeps the colours", () => {
    const parts = linesOf("fn run(a)", [{ line: 0, start: 0, end: 5, class: "keyword" }], {
      start: 3,
      end: 6,
    })[0];
    expect(parts?.map((part) => part.text).join("")).toBe("fn run(a)");
    expect(parts?.filter((part) => part.marked === true).map((part) => part.text)).toEqual([
      "ru",
      "n",
    ]);
    expect(parts?.find((part) => part.text === "ru")?.syntax).toBe("keyword");
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

  it("does not remember a failed call", async () => {
    run.mockRejectedValueOnce(new Error("no server"));
    const first = render(() => <Code path="a.rs" text="fn run()" />);
    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
    expect(cacheSize()).toBe(0);
    first.unmount();
    run.mockResolvedValue({
      pieces: [{ tokens: [{ line: 0, start: 0, end: 2, class: "keyword" }] }],
    });
    render(() => <Code path="a.rs" text="fn run()" />);
    await waitFor(() => expect(screen.getByText("fn").tagName).toBe("SPAN"));
    expect(run).toHaveBeenCalledTimes(2);
  });

  it("keeps at most CACHE_LIMIT answers", async () => {
    run.mockImplementation((_id: string, input: { pieces: unknown[] }) =>
      Promise.resolve({ pieces: input.pieces.map(() => ({ tokens: [] })) }),
    );
    const texts = Array.from({ length: CACHE_LIMIT + 10 }, (_, index) => `fn f${index}()`);
    render(() => <For each={texts}>{(text) => <Code path="a.rs" text={text} />}</For>);
    await waitFor(() => expect(cacheSize()).toBe(CACHE_LIMIT));
  });

  it("stays plain, and keeps nothing, when the answer has no pieces", async () => {
    run.mockResolvedValue({});
    render(() => <Code path="a.rs" text="fn run()" />);
    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
    expect(screen.getByText("fn run()")).toBeInTheDocument();
    expect(cacheSize()).toBe(0);
  });

  it("asks for all pieces of one moment in a single call", async () => {
    run.mockImplementation((_id: string, input: { pieces: unknown[] }) =>
      Promise.resolve({ pieces: input.pieces.map(() => ({ tokens: [] })) }),
    );
    const texts = Array.from({ length: 150 }, (_, index) => `fn row${index}()`);
    render(() => <For each={texts}>{(text) => <Code path="a.rs" text={text} />}</For>);
    await waitFor(() => expect(run).toHaveBeenCalledTimes(1));
    expect(run.mock.calls[0]?.[1].pieces).toHaveLength(150);
  });
});
