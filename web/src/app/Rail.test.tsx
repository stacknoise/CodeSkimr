import { render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { setLanguage } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import { EXPLORER_ROUTE, GRAPH_ROUTE, PROJECTS_ROUTE, SCAN_ROUTE, SETTINGS_ROUTE } from "./consts";
import { RailLink } from "./Frame";

afterEach(() => {
  openProject(null as unknown as number);
});

describe("RailLink", () => {
  it("shuts the screens of a project while none is open, but not projects and settings", () => {
    setLanguage("en");
    render(() => (
      <>
        <RailLink route={PROJECTS_ROUTE} />
        <RailLink route={EXPLORER_ROUTE} />
        <RailLink route={GRAPH_ROUTE} />
        <RailLink route={SCAN_ROUTE} />
        <RailLink route={SETTINGS_ROUTE} />
      </>
    ));
    expect(screen.getByRole("link", { name: "Projects" })).toHaveAttribute("href");
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute("href");
    for (const name of ["Explorer", "Dependencies", "Scans and performance"]) {
      expect(screen.getByRole("link", { name })).toHaveAttribute("aria-disabled", "true");
    }
  });

  it("opens the screens once a project is open, and every button has a tooltip", () => {
    setLanguage("en");
    openProject(1);
    render(() => <RailLink route={EXPLORER_ROUTE} />);
    const link = screen.getByRole("link", { name: "Explorer" });
    expect(link).toHaveAttribute("href", EXPLORER_ROUTE.path);
    expect(link).toHaveAttribute("title", "Explorer");
  });
});
