import { render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { t } from "../shared/i18n/i18n";
import { openProject } from "../shared/project/active";
import { FOOTER_ROUTES, MAIN_ROUTES } from "./consts";
import { RailLink } from "./Frame";

afterEach(() => {
  openProject(null as unknown as number);
});

describe("the buttons of the rail", () => {
  it("each carry their name as hover text and as accessible name", () => {
    openProject(1);
    for (const route of [...MAIN_ROUTES, ...FOOTER_ROUTES]) {
      const { unmount } = render(() => <RailLink route={route} />);
      const link = screen.getByRole("link", { name: t(route.label) });
      expect(link.getAttribute("title")).toBe(t(route.label));
      unmount();
    }
  });
});
