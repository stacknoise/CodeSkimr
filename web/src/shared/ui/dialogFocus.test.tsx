import { fireEvent, render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { useDialogFocus } from "./dialogFocus";

function Sample() {
  let box: HTMLDivElement | undefined;
  useDialogFocus(() => box);
  return (
    <div ref={box}>
      <button type="button">first</button>
      <input aria-label="field" />
      <button type="button">last</button>
      <button type="button" disabled>
        disabled
      </button>
      <button type="button" tabIndex={-1}>
        skipped
      </button>
    </div>
  );
}

describe("useDialogFocus", () => {
  it("wraps Tab at the last reachable field, ignoring disabled and skipped buttons", () => {
    render(() => <Sample />);
    const first = screen.getByRole("button", { name: "first" });
    const last = screen.getByRole("button", { name: "last" });
    last.focus();
    fireEvent.keyDown(document, { key: "Tab" });
    expect(first).toHaveFocus();
    fireEvent.keyDown(document, { key: "Tab", shiftKey: true });
    expect(last).toHaveFocus();
  });
});
