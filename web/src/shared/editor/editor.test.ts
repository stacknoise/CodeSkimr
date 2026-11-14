import { describe, expect, it, vi } from "vitest";
import { buildEditorUrl, encodePath, isEditorShortcut, openLink } from "./editor";

const config = {
  url_template: "vscode://file/{path}:{line}",
  allowed_schemes: ["vscode", "idea"],
};

describe("buildEditorUrl", () => {
  it("fills in path and line", () => {
    expect(buildEditorUrl(config, "/home/anton/src/a.rs", 42)).toBe(
      "vscode://file/home/anton/src/a.rs:42",
    );
  });

  it("keeps the leading slash when the template has none before the path", () => {
    expect(
      buildEditorUrl({ ...config, url_template: "vscode://file{path}" }, "/home/a.rs", 1),
    ).toBe("vscode://file/home/a.rs");
    expect(buildEditorUrl(config, "C:\\Users\\Anna\\a.cs", 3)).toBe(
      "vscode://file/C:/Users/Anna/a.cs:3",
    );
  });

  it("percent-encodes every part of the path but keeps the slashes", () => {
    expect(encodePath("src/my file#1.cs")).toBe("src/my%20file%231.cs");
    expect(encodePath("a/../b?x=1&y")).toBe("a/..%2Fb%3Fx%3D1%26y".replace("..%2F", "../"));
  });

  it("writes a Windows path with forward slashes and keeps the drive colon", () => {
    expect(encodePath("C:\\Users\\Anna Lee\\a.cs")).toBe("C:/Users/Anna%20Lee/a.cs");
  });

  it("shows line 1 when the line is unknown or not a line", () => {
    for (const line of [undefined, 0, -3, 1.5, Number.NaN]) {
      expect(buildEditorUrl(config, "a.rs", line)).toBe("vscode://file/a.rs:1");
    }
  });

  it("refuses a scheme that is not on the list", () => {
    const evil = { ...config, url_template: "javascript:alert({path})" };
    expect(buildEditorUrl(evil, "a.rs", 1)).toBeNull();
    const file = { ...config, url_template: "file:///{path}" };
    expect(buildEditorUrl(file, "a.rs", 1)).toBeNull();
  });

  it("compares schemes without regard to case", () => {
    const upper = { url_template: "VSCode://file/{path}", allowed_schemes: ["VSCODE"] };
    expect(buildEditorUrl(upper, "a.rs")).toBe("VSCode://file/a.rs");
  });

  it("refuses a template without scheme or without a place for the path", () => {
    expect(buildEditorUrl({ ...config, url_template: "file/{path}" }, "a.rs")).toBeNull();
    expect(buildEditorUrl({ ...config, url_template: "//vscode:{path}" }, "a.rs")).toBeNull();
    expect(buildEditorUrl({ ...config, url_template: "vscode://file/x" }, "a.rs")).toBeNull();
  });

  it("cannot be made to leave the scheme through the path", () => {
    const url = buildEditorUrl(config, "x:1\njavascript:alert(1)", 1);
    expect(url?.startsWith("vscode://")).toBe(true);
    expect(url).not.toContain("\n");
  });
});

describe("openLink", () => {
  it("hands the link to the browser", () => {
    const assign = vi.fn();
    openLink("vscode://file/a.rs:1", { assign });
    expect(assign).toHaveBeenCalledWith("vscode://file/a.rs:1");
  });
});

describe("isEditorShortcut", () => {
  const press = (init: KeyboardEventInit, target?: HTMLElement) => {
    const event = new KeyboardEvent("keydown", { ...init, bubbles: true });
    if (target) {
      Object.defineProperty(event, "target", { value: target });
    }
    return isEditorShortcut(event);
  };

  it("accepts E and Alt+Enter", () => {
    expect(press({ key: "e" })).toBe(true);
    expect(press({ key: "E" })).toBe(true);
    expect(press({ key: "Enter", altKey: true })).toBe(true);
  });

  it("ignores other keys and combinations", () => {
    expect(press({ key: "Enter" })).toBe(false);
    expect(press({ key: "e", ctrlKey: true })).toBe(false);
    expect(press({ key: "e", metaKey: true })).toBe(false);
    expect(press({ key: "e", altKey: true })).toBe(false);
    expect(press({ key: "x" })).toBe(false);
  });

  it("ignores typing in a field", () => {
    expect(press({ key: "e" }, document.createElement("input"))).toBe(false);
    expect(press({ key: "e" }, document.createElement("textarea"))).toBe(false);
  });
});
