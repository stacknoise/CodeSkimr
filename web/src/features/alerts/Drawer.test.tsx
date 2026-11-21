import { fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { openProject } from "../../shared/project/active";
import { alertPage, CHANGED, DISMISSED, ORPHANED, REMOVED } from "../../tests/alerts";
import { bodiesOf, serveBodies } from "../../tests/annotations";
import { answer, useServerSession } from "../../tests/commands";
import { Bell } from "./Bell";
import { AlertsDrawer } from "./Drawer";
import { refreshCount, resetAlerts } from "./store";

useServerSession();

function serveAlerts(open = [CHANGED, REMOVED, ORPHANED]) {
  serveBodies({
    "alerts.list": (body) =>
      answer(
        body.state === "dismissed"
          ? alertPage([DISMISSED], open.length)
          : alertPage(open, open.length),
      ),
    "alerts.dismiss": () => answer({ dismissed: 1, open: 2 }),
    "alerts.dismiss_all": () => answer({ dismissed: 3, open: 0 }),
    "annotations.update": () => answer({ id: 12 }),
    "annotations.delete": () => answer({ id: 12 }),
    "annotations.summary": () =>
      answer({ items: [], total: 0, offset: 0, limit: 500, next_offset: null }),
  });
}

async function renderDrawer() {
  render(() => (
    <>
      <Bell />
      <AlertsDrawer />
    </>
  ));
  await refreshCount(1);
}

beforeEach(() => openProject(1));
afterEach(resetAlerts);

describe("the bell", () => {
  it("says how many alerts are open, in its label", async () => {
    serveAlerts();
    await renderDrawer();
    const bell = await screen.findByRole("button", { name: "Alerts, 3 open" });
    expect(bell.getAttribute("aria-expanded")).toBe("false");
    expect(within(bell).getByText("3")).toBeTruthy();
  });

  it("is not drawn on a server without alerts", async () => {
    serveBodies({});
    await renderDrawer();
    expect(screen.queryByRole("button", { name: /Alerts/ })).toBeNull();
  });
});

describe("the drawer", () => {
  it("opens from the bell with the tabs and their counters, and shows the alerts as text", async () => {
    serveAlerts();
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 3 open" }));
    const tabs = await screen.findByRole("tablist", { name: "Kinds of alerts" });
    expect(await within(tabs).findByRole("tab", { name: "Open 3" })).toBeTruthy();
    expect(within(tabs).getByRole("tab", { name: "Changed 1" })).toBeTruthy();
    expect(within(tabs).getByRole("tab", { name: "Removed 2" })).toBeTruthy();
    expect(within(tabs).getByRole("tab", { name: "Dismissed 1" })).toBeTruthy();
    expect(await screen.findByText("CHANGED")).toBeTruthy();
    expect(screen.getByText("Changed: complexity, lines of code.")).toBeTruthy();
    expect(
      screen.getByText("The symbol no longer exists. 4 places still refer to it."),
    ).toBeTruthy();
  });

  it("moves between the tabs with the arrow keys and lists the orphaned targets under Removed", async () => {
    serveAlerts();
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 3 open" }));
    const open = await screen.findByRole("tab", { name: "Open 3" });
    fireEvent.keyDown(open, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Changed 1" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    fireEvent.keyDown(screen.getByRole("tab", { name: "Changed 1" }), { key: "ArrowRight" });
    expect(await screen.findByText("ORPHANED")).toBeTruthy();
    expect(screen.getByText("REMOVED")).toBeTruthy();
    expect(screen.queryByText("CHANGED")).toBeNull();
  });

  it("closes with Escape and gives the focus back to the bell", async () => {
    serveAlerts();
    await renderDrawer();
    const bell = await screen.findByRole("button", { name: "Alerts, 3 open" });
    fireEvent.click(bell);
    const drawer = await screen.findByRole("dialog", { name: "Alerts" });
    fireEvent.keyDown(drawer, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(document.activeElement).toBe(bell);
  });

  it("dismisses one alert with its cross", async () => {
    serveAlerts();
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 3 open" }));
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Dismiss alert: Shop.Orders.IOrderRepository.Save",
      }),
    );
    await waitFor(() => expect(bodiesOf("alerts.dismiss")).toEqual([{ project: 1, alert: 2 }]));
  });

  it("asks before it dismisses all alerts of a tab", async () => {
    serveAlerts();
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 3 open" }));
    fireEvent.click(await screen.findByRole("button", { name: "Dismiss all open alerts" }));
    expect(bodiesOf("alerts.dismiss_all")).toHaveLength(0);
    expect(screen.getByText("Dismiss all 3 alerts of this tab?")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Yes, dismiss all" }));
    await waitFor(() =>
      expect(bodiesOf("alerts.dismiss_all")).toEqual([{ project: 1, kinds: [], confirm: true }]),
    );
  });

  it("offers an orphaned target to be reassigned to the closest symbol or to lose its mark", async () => {
    serveAlerts([ORPHANED]);
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 1 open" }));
    fireEvent.click(await screen.findByRole("button", { name: "Reassign" }));
    await waitFor(() =>
      expect(bodiesOf("annotations.update")).toEqual([
        {
          project: 1,
          id: 12,
          target: { kind: "method", name: "Shop.Orders.OrderService.RecalculateTotal" },
        },
      ]),
    );
    fireEvent.click(await screen.findByRole("button", { name: "Remove mark" }));
    await waitFor(() => expect(bodiesOf("annotations.delete")).toEqual([{ project: 1, id: 12 }]));
  });

  it("writes a reason as text, never as markup", async () => {
    serveAlerts([{ ...CHANGED, reason: "<img src=x onerror=alert(1)>" }]);
    await renderDrawer();
    fireEvent.click(await screen.findByRole("button", { name: "Alerts, 1 open" }));
    expect(await screen.findByText("Reason: <img src=x onerror=alert(1)>")).toBeTruthy();
    expect(document.querySelector("img")).toBeNull();
  });
});
