import { fireEvent, render, screen } from "@solidjs/testing-library";
import { ErrorBoundary } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../shared/i18n/i18n";
import { UnexpectedError, detailsOf } from "./UnexpectedError";

function Broken(): never {
  throw new Error("boom");
}

function renderBroken() {
  return render(() => (
    <ErrorBoundary fallback={(error, reset) => <UnexpectedError error={error} reset={reset} />}>
      <Broken />
    </ErrorBoundary>
  ));
}

afterEach(() => {
  setLanguage("en");
  vi.restoreAllMocks();
});

describe("UnexpectedError", () => {
  it("shows the message, the code and the buttons instead of the broken screen", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    renderBroken();
    expect(screen.getByRole("alert").textContent).toContain("An unexpected error occurred");
    expect(screen.getByText("UI-901")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Copy details" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy();
  });

  it("speaks German", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    setLanguage("de");
    renderBroken();
    expect(screen.getByRole("button", { name: "Details kopieren" })).toBeTruthy();
  });

  it("copies the code and the exception, without a stack", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    renderBroken();
    fireEvent.click(screen.getByRole("button", { name: "Copy details" }));
    await vi.waitFor(() => expect(screen.getByRole("button", { name: "Copied" })).toBeTruthy());
    const copied = String(writeText.mock.calls[0]);
    expect(copied).toContain("code=UI-901");
    expect(copied).toContain("Error: boom");
    expect(copied).not.toContain("at ");
    vi.unstubAllGlobals();
  });

  it("shows the details to select when there is no clipboard", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    vi.stubGlobal("navigator", {});
    renderBroken();
    fireEvent.click(screen.getByRole("button", { name: "Copy details" }));
    const box = screen.getByRole("textbox", { name: "Details to copy" }) as HTMLTextAreaElement;
    expect(box.value).toContain("code=UI-901");
    vi.unstubAllGlobals();
  });

  it("shows the details to select when the browser refuses to copy", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const writeText = vi.fn(() => Promise.reject(new Error("denied")));
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    renderBroken();
    fireEvent.click(screen.getByRole("button", { name: "Copy details" }));
    await vi.waitFor(() => expect(screen.getByRole("textbox")).toBeTruthy());
    vi.unstubAllGlobals();
  });

  it("names a thrown value that is no exception", () => {
    expect(detailsOf("text", "/x")).toBe("code=UI-901\n/x\ntext");
  });
});
