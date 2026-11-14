import { fireEvent, render } from "@solidjs/testing-library";
import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { cursorTarget, onTreeKey, useExplorerKeys } from "./keys";
import type { TreeRow } from "./tree";
import { useWindow } from "./useWindow";

const NO_CURSOR = -1;
const LONG_LIST = 1000;
const ROW_HEIGHT = 26;
const MOST_ROWS = 100;

const row = (key: string, extra: Partial<TreeRow> = {}): TreeRow => ({
  key,
  kind: "type",
  depth: 0,
  label: key,
  meta: "",
  expandable: false,
  open: false,
  type: null,
  member: null,
  symbolKind: null,
  layerIndex: null,
  ...extra,
});

const ROWS = [row("a", { expandable: true }), row("b", { expandable: true, open: true }), row("c")];

function press(key: string, cursor: number, modifiers: KeyboardEventInit = {}) {
  const actions = { onCursor: vi.fn(), onToggle: vi.fn(), onActivate: vi.fn() };
  const event = new KeyboardEvent("keydown", { key, cancelable: true, ...modifiers });
  onTreeKey(event, ROWS, cursor, actions);
  return { actions, event };
}

describe("cursorTarget", () => {
  it("moves by one and stops at the ends", () => {
    expect(cursorTarget("ArrowDown", 0, 3)).toBe(1);
    expect(cursorTarget("ArrowDown", 2, 3)).toBe(2);
    expect(cursorTarget("ArrowUp", 1, 3)).toBe(0);
    expect(cursorTarget("ArrowUp", 0, 3)).toBe(0);
  });

  it("jumps with Home and End and ignores other keys", () => {
    expect(cursorTarget("Home", 2, 3)).toBe(0);
    expect(cursorTarget("End", 0, 3)).toBe(2);
    expect(cursorTarget("x", 0, 3)).toBeNull();
  });

  it("starts at the first row when there is no cursor yet", () => {
    expect(cursorTarget("ArrowDown", NO_CURSOR, 3)).toBe(0);
    expect(cursorTarget("ArrowUp", NO_CURSOR, 3)).toBe(0);
  });
});

describe("onTreeKey", () => {
  it("moves the cursor and keeps the page from scrolling", () => {
    const { actions, event } = press("ArrowDown", 0);
    expect(actions.onCursor).toHaveBeenCalledWith("b");
    expect(event.defaultPrevented).toBe(true);
  });

  it("opens a closed row with Right and closes an open one with Left", () => {
    expect(press("ArrowRight", 0).actions.onToggle).toHaveBeenCalledWith(ROWS[0]);
    expect(press("ArrowRight", 1).actions.onToggle).not.toHaveBeenCalled();
    expect(press("ArrowLeft", 1).actions.onToggle).toHaveBeenCalledWith(ROWS[1]);
    expect(press("ArrowLeft", 0).actions.onToggle).not.toHaveBeenCalled();
  });

  it("acts like a click on Enter and Space", () => {
    expect(press("Enter", 2).actions.onActivate).toHaveBeenCalledWith(ROWS[2]);
    expect(press(" ", 2).actions.onActivate).toHaveBeenCalledWith(ROWS[2]);
  });

  it("leaves other keys and the keys with a modifier alone", () => {
    const other = press("x", 0);
    expect(other.event.defaultPrevented).toBe(false);
    const control = press("ArrowDown", 0, { ctrlKey: true });
    expect(control.actions.onCursor).not.toHaveBeenCalled();
    expect(control.event.defaultPrevented).toBe(false);
  });
});

describe("useExplorerKeys", () => {
  function shown() {
    const keys = {
      collapseAll: vi.fn(),
      reveal: vi.fn(),
      usages: vi.fn(),
      graph: vi.fn(),
      editor: vi.fn(),
    };
    const view = render(() => {
      useExplorerKeys(keys);
      return <input aria-label="field" />;
    });
    return { keys, view };
  }

  it("closes everything on Ctrl+Left and shows the selection on Ctrl+.", () => {
    const { keys } = shown();
    fireEvent.keyDown(document.body, { key: "ArrowLeft", ctrlKey: true });
    fireEvent.keyDown(document.body, { key: ".", ctrlKey: true });
    expect(keys.collapseAll).toHaveBeenCalledOnce();
    expect(keys.reveal).toHaveBeenCalledOnce();
  });

  it("shows the usages on U, but not with a modifier or while the user types", () => {
    const { keys, view } = shown();
    fireEvent.keyDown(document.body, { key: "u" });
    fireEvent.keyDown(document.body, { key: "u", ctrlKey: true });
    fireEvent.keyDown(view.getByLabelText("field"), { key: "u" });
    expect(keys.usages).toHaveBeenCalledOnce();
  });

  it("shows the map on G, but not with a modifier or while the user types", () => {
    const { keys, view } = shown();
    fireEvent.keyDown(document.body, { key: "g" });
    fireEvent.keyDown(document.body, { key: "g", ctrlKey: true });
    fireEvent.keyDown(view.getByLabelText("field"), { key: "g" });
    expect(keys.graph).toHaveBeenCalledOnce();
    expect(keys.usages).not.toHaveBeenCalled();
  });

  it("opens the editor on E and Alt+Enter, but not with Ctrl or while the user types", () => {
    const { keys, view } = shown();
    fireEvent.keyDown(document.body, { key: "e" });
    fireEvent.keyDown(document.body, { key: "Enter", altKey: true });
    fireEvent.keyDown(document.body, { key: "e", ctrlKey: true });
    fireEvent.keyDown(view.getByLabelText("field"), { key: "e" });
    expect(keys.editor).toHaveBeenCalledTimes(2);
  });

  it("ignores the keys without Ctrl and while the user types", () => {
    const { keys, view } = shown();
    fireEvent.keyDown(document.body, { key: "ArrowLeft" });
    fireEvent.keyDown(view.getByLabelText("field"), { key: ".", ctrlKey: true });
    expect(keys.collapseAll).not.toHaveBeenCalled();
    expect(keys.reveal).not.toHaveBeenCalled();
  });
});

describe("useWindow", () => {
  it("keeps only a window of a long list and the length of the whole", () => {
    createRoot((dispose) => {
      const items = Array.from({ length: LONG_LIST }, (_, index) => index);
      const view = useWindow(
        () => items,
        () => NO_CURSOR,
      );
      expect(view.visible().length).toBeLessThan(MOST_ROWS);
      expect(view.visible()[0]?.index).toBe(0);
      expect(view.total()).toBe(LONG_LIST * ROW_HEIGHT);
      dispose();
    });
  });
});
