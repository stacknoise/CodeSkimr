import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { CodesScreen } from "./CodesScreen";

beforeEach(() => setLanguage("en"));

describe("CodesScreen", () => {
  it("lists the codes with their messages", () => {
    render(() => <CodesScreen />);
    expect(screen.getByRole("heading", { name: "Error codes" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "SRC-201" })).toBeTruthy();
  });

  it("narrows the list by the search text", () => {
    render(() => <CodesScreen />);
    fireEvent.input(screen.getByLabelText("Search error codes"), { target: { value: "PRJ-302" } });
    expect(screen.getByRole("button", { name: "PRJ-302" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "SRC-201" })).toBeNull();
  });

  it("opens the explanation of a code on a click", () => {
    render(() => <CodesScreen />);
    fireEvent.click(screen.getByRole("button", { name: "SRC-202" }));
    expect(screen.getByText(/Developer description/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Copy details" })).toBeTruthy();
  });

  it("shows a retired code only for its full id, marked as retired", () => {
    render(() => <CodesScreen />);
    expect(screen.queryByRole("button", { name: "APP-301" })).toBeNull();
    fireEvent.input(screen.getByLabelText("Search error codes"), { target: { value: "app-301" } });
    fireEvent.click(screen.getByRole("button", { name: "APP-301" }));
    expect(screen.getByText(/This code is retired/)).toBeTruthy();
  });

  it("says so when nothing matches", () => {
    render(() => <CodesScreen />);
    fireEvent.input(screen.getByLabelText("Search error codes"), { target: { value: "zzzz" } });
    expect(screen.getByText("No code matches.")).toBeTruthy();
  });
});
