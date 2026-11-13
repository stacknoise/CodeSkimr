import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { SettingsScreen } from "./SettingsScreen";

beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("matchMedia", () => ({ matches: false }));
  document.documentElement.setAttribute("data-theme", "petrol");
  setLanguage("en");
});

describe("SettingsScreen", () => {
  it("offers the three themes with a preview each and marks the active one", () => {
    render(() => <SettingsScreen />);
    expect(screen.getByText("Light")).toBeInTheDocument();
    expect(screen.getByText("Petrol")).toBeInTheDocument();
    expect(screen.getByText("Dark")).toBeInTheDocument();
    expect(document.querySelectorAll("[data-theme]").length).toBeGreaterThanOrEqual(3);
    expect(screen.getAllByText("Selected")).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Select" })).toHaveLength(2);
  });

  it("applies a theme at once and remembers it", async () => {
    render(() => <SettingsScreen />);
    const buttons = screen.getAllByRole("button", { name: "Select" });
    fireEvent.click(buttons[buttons.length - 1] as HTMLElement);
    expect(document.documentElement.getAttribute("data-theme")).toBe("dunkel");
    expect(localStorage.getItem("codeskimr.theme")).toBe("dunkel");
    expect(screen.getAllByText("Selected")).toHaveLength(1);
  });

  it("goes back to the system setting", () => {
    localStorage.setItem("codeskimr.theme", "hell");
    render(() => <SettingsScreen />);
    const box = screen.getByRole("checkbox");
    fireEvent.click(box);
    expect(localStorage.getItem("codeskimr.theme")).toBeNull();
  });

  it("switches the language of the screen", () => {
    render(() => <SettingsScreen />);
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "de" } });
    expect(screen.getByText("Einstellungen")).toBeInTheDocument();
    expect(localStorage.getItem("codeskimr.language")).toBe("de");
  });
});
