import { render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { setLanguage, t } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import {
  EXPLORER_ROUTE,
  FOOTER_ROUTES,
  MAIN_ROUTES,
  PROJECTS_ROUTE,
  SETTINGS_ROUTE,
} from "./consts";
import { RailLink } from "./Frame";

afterEach(() => {
  openProject(null as unknown as number);
});

describe("RailLink", () => {
  it("shuts every screen of the rail while no project is open, but not projects and settings", () => {
    setLanguage("en");
    const open = [PROJECTS_ROUTE.path, SETTINGS_ROUTE.path];
    for (const route of [...MAIN_ROUTES, ...FOOTER_ROUTES]) {
      const { unmount } = render(() => <RailLink route={route} />);
      const link = screen.getByRole("link", { name: new RegExp(`^${t(route.label)}`) });
      if (open.includes(route.path)) {
        expect(link).toHaveAttribute("href");
        expect(link).not.toHaveAttribute("aria-disabled");
      } else {
        expect(link).toHaveAttribute("aria-disabled", "true");
      }
      unmount();
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
