import { describe, expect, it } from "vitest";
import { isSafeLink, parseMarkup } from "./markup";

describe("parseMarkup", () => {
  it("keeps plain text as one piece", () => {
    expect(parseMarkup("plain text")).toEqual([{ kind: "text", text: "plain text" }]);
    expect(parseMarkup("")).toEqual([]);
  });

  it("reads bold, italic and code", () => {
    expect(parseMarkup("a **b** *c* `d` e")).toEqual([
      { kind: "text", text: "a " },
      { kind: "bold", text: "b" },
      { kind: "text", text: " " },
      { kind: "italic", text: "c" },
      { kind: "text", text: " " },
      { kind: "code", text: "d" },
      { kind: "text", text: " e" },
    ]);
  });

  it("reads a link with a text and a bare address, both http and https", () => {
    expect(parseMarkup("[docs](https://example.org/a) http://example.org/b")).toEqual([
      { kind: "link", text: "docs", href: "https://example.org/a" },
      { kind: "text", text: " " },
      { kind: "link", text: "http://example.org/b", href: "http://example.org/b" },
    ]);
  });

  it("makes no link of any other scheme", () => {
    const pieces = parseMarkup(
      "[x](javascript:alert(1)) data:text/html,<b>x</b> [y](ftp://host/f)",
    );
    expect(pieces.every((piece) => piece.kind !== "link")).toBe(true);
    expect(pieces.map((piece) => piece.text).join("")).toContain("javascript:alert(1)");
  });

  it("leaves markup characters of HTML as text", () => {
    expect(parseMarkup("<img src=x onerror=alert(1)> & more")).toEqual([
      { kind: "text", text: "<img src=x onerror=alert(1)> & more" },
    ]);
  });

  it("leaves an unclosed marker as text", () => {
    expect(parseMarkup("**open and `open")).toEqual([{ kind: "text", text: "**open and `open" }]);
  });
});

describe("isSafeLink", () => {
  it("accepts http and https and nothing else", () => {
    expect(isSafeLink("https://example.org")).toBe(true);
    expect(isSafeLink("http://example.org")).toBe(true);
    expect(isSafeLink("javascript:alert(1)")).toBe(false);
    expect(isSafeLink("not a url")).toBe(false);
  });
});
