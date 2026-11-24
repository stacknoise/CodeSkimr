import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";
import { useClipboard } from "../../tests/clipboard";
import { NoticeBox } from "./NoticeBox";

describe("the details of an error note", () => {
  const setClipboard = useClipboard();

  it("shows the code and copies the code with the message", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    setClipboard({ writeText });
    render(() => <NoticeBox text="Der Ordner fehlt." code="SRC-201" alert />);
    expect(screen.getByText("SRC-201")).toBeTruthy();
    fireEvent.click(screen.getByRole("button"));
    expect(writeText).toHaveBeenCalledWith("SRC-201: Der Ordner fehlt.");
  });

  it("shows the details in a box where there is no clipboard", () => {
    setClipboard(undefined);
    render(() => <NoticeBox text="Der Ordner fehlt." code="SRC-201" alert />);
    fireEvent.click(screen.getByRole("button"));
    expect((screen.getByRole("textbox") as HTMLTextAreaElement).value).toBe(
      "SRC-201: Der Ordner fehlt.",
    );
  });

  it("shows the details in a box where the browser refuses to copy", async () => {
    setClipboard({ writeText: vi.fn().mockRejectedValue(new Error("refused")) });
    render(() => <NoticeBox text="Der Ordner fehlt." code="SRC-201" alert />);
    fireEvent.click(screen.getByRole("button"));
    await waitFor(() => expect(screen.getByRole("textbox")).toBeTruthy());
  });

  it("shows no button for a note without a code", () => {
    render(() => <NoticeBox text="Wird geladen." />);
    expect(screen.queryByRole("button")).toBeNull();
  });
});
