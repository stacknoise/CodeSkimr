import { render, screen } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { t } from "../shared/i18n/i18n";
import { FOOTER_ROUTES, MAIN_ROUTES } from "./consts";
import { RailLink } from "./Frame";

describe("the buttons of the rail", () => {
  it("each carry their name as hover text and as accessible name", () => {
    for (const route of [...MAIN_ROUTES, ...FOOTER_ROUTES]) {
      const { unmount } = render(() => <RailLink route={route} />);
      const link = screen.getByRole("link", { name: t(route.label) });
      expect(link.getAttribute("title")).toBe(t(route.label));
      unmount();
    }
  });
});
