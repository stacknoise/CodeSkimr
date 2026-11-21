import { fireEvent, render, screen } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { NoticeBox } from "./NoticeBox";

describe("the details of an error note", () => {
  it("shows the code and copies the code with the message", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(() => <NoticeBox text="Der Ordner fehlt." code="SRC-201" alert />);
    expect(screen.getByText("SRC-201")).toBeTruthy();
    fireEvent.click(screen.getByRole("button"));
    expect(writeText).toHaveBeenCalledWith("SRC-201: Der Ordner fehlt.");
  });

  it("shows no button for a note without a code", () => {
    render(() => <NoticeBox text="Wird geladen." />);
    expect(screen.queryByRole("button")).toBeNull();
  });
});
