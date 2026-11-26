import { fireEvent, render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { setLanguage } from "../../shared/i18n/i18n";
import { currentPath } from "../../app/router";
import { answer, serve, useServerSession } from "../../tests/commands";
import { refreshStatus } from "../status/status";
import { SETTINGS_TAB } from "./consts";
import { SettingsScreen } from "./SettingsScreen";
import { setEffectiveSearch } from "./effective";
import { settingsTab, showSettingsTab } from "./tab";

beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("matchMedia", () => ({ matches: false }));
  document.documentElement.setAttribute("data-theme", "petrol");
  setLanguage("en");
  showSettingsTab(SETTINGS_TAB.appearance);
  setEffectiveSearch("");
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

  it("leads to the layer editor", () => {
    render(() => <SettingsScreen />);
    fireEvent.click(screen.getByRole("tab", { name: "Analysis" }));
    fireEvent.click(screen.getByRole("button", { name: "Edit layers" }));
    expect(currentPath()).toBe("/layers");
  });
});

describe("the tabs of the settings screen", () => {
  useServerSession();

  it("shows five tabs and the appearance first", () => {
    render(() => <SettingsScreen />);
    const names = screen.getAllByRole("tab").map((tab) => tab.textContent);
    expect(names).toEqual(["Appearance", "Analysis", "Access", "Effective settings", "Info"]);
    expect(screen.getByRole("tab", { name: "Appearance" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByText("Theme")).toBeInTheDocument();
    expect(screen.queryByText("Edit layers")).toBeNull();
  });

  it("moves along the tabs with the arrow keys", () => {
    render(() => <SettingsScreen />);
    fireEvent.keyDown(screen.getByRole("tablist"), { key: "ArrowRight" });
    expect(settingsTab()).toBe(SETTINGS_TAB.analysis);
    fireEvent.keyDown(screen.getByRole("tablist"), { key: "ArrowLeft" });
    fireEvent.keyDown(screen.getByRole("tablist"), { key: "ArrowLeft" });
    expect(settingsTab()).toBe(SETTINGS_TAB.info);
  });

  it("shows the effective settings when the search is used on another tab", () => {
    serve({ "settings.effective": () => answer({ user_file: null, overrides: [], settings: [] }) });
    render(() => <SettingsScreen />);
    fireEvent.input(screen.getByLabelText("Search a setting …"), { target: { value: "port" } });
    expect(settingsTab()).toBe(SETTINGS_TAB.effective);
    expect(screen.getByRole("tab", { name: "Effective settings" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("clears the search text when another tab is chosen", () => {
    serve({ "settings.effective": () => answer({ user_file: null, overrides: [], settings: [] }) });
    render(() => <SettingsScreen />);
    fireEvent.input(screen.getByLabelText("Search a setting …"), { target: { value: "port" } });
    fireEvent.click(screen.getByRole("tab", { name: "Appearance" }));
    expect(screen.getByLabelText("Search a setting …")).toHaveValue("");
  });

  it("keeps the configuration file and the error codes on the tab Info", async () => {
    serve({ "settings.effective": () => answer({ user_file: null, overrides: [], settings: [] }) });
    render(() => <SettingsScreen />);
    fireEvent.click(screen.getByRole("tab", { name: "Info" }));
    expect(screen.getByRole("button", { name: "Show error codes" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Configuration" })).toBeInTheDocument();
  });
});

describe("the version on the settings screen", () => {
  useServerSession();

  it("is the one that codeskimr --version prints", async () => {
    serve({ "status.info": () => answer({ version: "1.2.3" }) });
    await refreshStatus();
    render(() => <SettingsScreen />);
    expect(screen.getByText("CodeSkimr version 1.2.3")).toBeInTheDocument();
  });
});
