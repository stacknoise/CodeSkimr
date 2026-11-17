import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { openProject } from "../../shared/project/active";
import { alertPage, CHANGED, DISMISSED, ORPHANED, REMOVED } from "../../tests/alerts";
import { callsOf, bodiesOf, serveBodies } from "../../tests/annotations";
import { answer, useServerSession } from "../../tests/commands";
import { TAB } from "./consts";
import {
  alertsOfTab,
  applyAlertsEvent,
  dismiss,
  dismissKinds,
  doneAlerts,
  loadAlerts,
  openAlertCount,
  openAlerts,
  parseAlertsEvent,
  refreshCount,
  resetAlerts,
} from "./store";

useServerSession();
beforeEach(() => openProject(1));
afterEach(resetAlerts);

/** The server of a project with three open alerts and one dismissed. */
function serveAlerts() {
  serveBodies({
    "alerts.list": (body) =>
      answer(
        body.state === "dismissed"
          ? alertPage([DISMISSED], 3)
          : alertPage([CHANGED, REMOVED, ORPHANED], 3),
      ),
    "alerts.dismiss": () => answer({ dismissed: 1, open: 2 }),
    "alerts.dismiss_all": () => answer({ dismissed: 2, open: 1 }),
  });
}

describe("the number on the bell", () => {
  it("is read with one call for one row", async () => {
    serveAlerts();
    await refreshCount(1);
    expect(openAlertCount()).toBe(3);
    expect(bodiesOf("alerts.list")).toEqual([{ project: 1, state: "open", limit: 1 }]);
  });

  it("stays unknown on a server without alerts", async () => {
    serveBodies({});
    await refreshCount(1);
    expect(openAlertCount()).toBeNull();
  });

  it("follows the event of the stream and ignores one of another project", () => {
    serveAlerts();
    applyAlertsEvent(2, 9);
    expect(openAlertCount()).toBeNull();
    applyAlertsEvent(1, 4);
    expect(openAlertCount()).toBe(4);
  });
});

describe("the lists of the drawer", () => {
  it("read the open and the dismissed alerts, two calls", async () => {
    serveAlerts();
    await loadAlerts(1);
    expect(openAlerts()).toHaveLength(3);
    expect(doneAlerts()).toHaveLength(1);
    expect(callsOf("alerts.list")).toBe(2);
  });

  it("sort the open alerts into the tabs, orphaned targets with the removed ones", () => {
    const open = [CHANGED, REMOVED, ORPHANED];
    expect(alertsOfTab(TAB.open, open, [DISMISSED])).toHaveLength(3);
    expect(alertsOfTab(TAB.changed, open, [DISMISSED])).toEqual([CHANGED]);
    expect(alertsOfTab(TAB.removed, open, [DISMISSED])).toEqual([REMOVED, ORPHANED]);
    expect(alertsOfTab(TAB.done, open, [DISMISSED])).toEqual([DISMISSED]);
  });
});

describe("dismissing", () => {
  it("sends the id, then reads the lists again", async () => {
    serveAlerts();
    await dismiss(1, 2);
    expect(bodiesOf("alerts.dismiss")).toEqual([{ project: 1, alert: 2 }]);
    expect(callsOf("alerts.list")).toBe(2);
  });

  it("confirms a bulk dismissal, as the server wants", async () => {
    serveAlerts();
    await dismissKinds(1, ["removed", "orphaned"]);
    expect(bodiesOf("alerts.dismiss_all")).toEqual([
      { project: 1, kinds: ["removed", "orphaned"], confirm: true },
    ]);
  });
});

describe("the event", () => {
  it("is read from the stream message", () => {
    expect(parseAlertsEvent('{"type":"alerts_changed","project":1,"open":5}')).toEqual({
      project: 1,
      open: 5,
    });
    expect(parseAlertsEvent('{"type":"scan_progressed","project":1}')).toBeNull();
    expect(parseAlertsEvent("not json")).toBeNull();
  });
});
