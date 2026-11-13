import { describe, expect, it } from "vitest";
import { pathOf, readDrop } from "./drop";

function data(values: Record<string, string>, files = 0): DataTransfer {
  return {
    getData: (type: string) => values[type] ?? "",
    files: { length: files },
    items: { length: files },
  } as unknown as DataTransfer;
}

describe("pathOf", () => {
  it("turns a file address into a path", () => {
    expect(pathOf("file:///home/me/my%20app")).toBe("/home/me/my app");
    expect(pathOf("file:///C:/work/app")).toBe("C:/work/app");
  });

  it("leaves other text as it is", () => {
    expect(pathOf("https://example.org/app.git")).toBe("https://example.org/app.git");
  });
});

describe("readDrop", () => {
  it("reads the first address of a list and skips its comments", () => {
    const dropped = readDrop(
      data({ "text/uri-list": "# comment\nfile:///work/app\nfile:///other" }),
    );
    expect(dropped).toEqual({ kind: "text", text: "/work/app" });
  });

  it("takes dropped text when there is no address", () => {
    expect(readDrop(data({ "text/plain": "https://example.org/app.git" }))).toEqual({
      kind: "text",
      text: "https://example.org/app.git",
    });
  });

  it("knows a folder without a path and an empty drop", () => {
    expect(readDrop(data({}, 1))).toEqual({ kind: "opaque" });
    expect(readDrop(data({}))).toEqual({ kind: "empty" });
  });
});
