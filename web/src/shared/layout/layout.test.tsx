import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { pointer } from "../../tests/pointer";
import { LAYOUT_STORAGE_PREFIX, ORIENTATION, SIDE } from "./consts";
import { createPaneSize } from "./paneSize";
import { Splitter } from "./Splitter";

const BOUNDS = { initial: 300, min: 200, max: 500 };

beforeEach(() => localStorage.clear());

describe("the size of a pane", () => {
  it("starts with the first value, keeps within the limits and remembers the choice", () => {
    const pane = createPaneSize("test.pane", BOUNDS);
    expect(pane.size()).toBe(300);
    pane.set(900);
    expect(pane.size()).toBe(500);
    pane.set(250);
    expect(localStorage.getItem(`${LAYOUT_STORAGE_PREFIX}test.pane`)).toBe("250");
    expect(createPaneSize("test.pane", BOUNDS).size()).toBe(250);
  });

  it("ignores a stored value that is no number and goes back on reset", () => {
    localStorage.setItem(`${LAYOUT_STORAGE_PREFIX}test.pane`, "wide");
    const pane = createPaneSize("test.pane", BOUNDS);
    expect(pane.size()).toBe(300);
    pane.set(400);
    pane.reset();
    expect(pane.size()).toBe(300);
    expect(localStorage.getItem(`${LAYOUT_STORAGE_PREFIX}test.pane`)).toBeNull();
  });
});

describe("the splitter", () => {
  function shown(side: typeof SIDE.before | typeof SIDE.after) {
    const pane = createPaneSize("test.pane", BOUNDS);
    render(() => <Splitter pane={pane} orientation={ORIENTATION.vertical} side={side} />);
    return { pane, handle: screen.getByRole("separator") };
  }

  it("grows a pane before it with the right arrow and a pane after it with the left one", () => {
    const before = shown(SIDE.before);
    fireEvent.keyDown(before.handle, { key: "ArrowRight" });
    expect(before.pane.size()).toBe(316);
  });

  it("shrinks a pane after it with the right arrow and goes to the ends with Home and End", () => {
    const after = shown(SIDE.after);
    fireEvent.keyDown(after.handle, { key: "ArrowRight", shiftKey: true });
    expect(after.pane.size()).toBe(236);
    fireEvent.keyDown(after.handle, { key: "Home" });
    expect(after.pane.size()).toBe(500);
    fireEvent.keyDown(after.handle, { key: "End" });
    expect(after.pane.size()).toBe(200);
  });

  it("follows the pointer while it is dragged and goes back on a double click", () => {
    const { pane, handle } = shown(SIDE.before);
    handle.setPointerCapture = () => undefined;
    pointer(handle, "pointerdown", { clientX: 100 });
    pointer(handle, "pointermove", { clientX: 160 });
    expect(pane.size()).toBe(360);
    pointer(handle, "pointerup");
    pointer(handle, "pointermove", { clientX: 300 });
    expect(pane.size()).toBe(360);
    fireEvent.dblClick(handle);
    expect(pane.size()).toBe(300);
  });
});
