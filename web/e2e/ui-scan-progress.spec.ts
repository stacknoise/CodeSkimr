// Starting a scan shows the scan screen, and the stages report their share (WP scan progress).
// The server is the real one; the rescan is held back and the events of the stream are sent by the
// test, so that the screen can be looked at in the middle of a scan. `UPDATE_SCREENSHOTS=1` writes
// the screens to docs/screenshots/scan-progress/.
import path from "node:path";
import type { Page, WebSocketRoute } from "@playwright/test";
import { expect, selectScannedProject, shot, showTab, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/scan-progress");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE =
  "namespace Prog;\n\npublic class Counter\n{\n    public int Next(int a) { return a + 1; }\n}\n";

interface Held {
  stream: WebSocketRoute | null;
  ids: Map<string, number>;
  release: () => void;
  /** Whether the next rescan is held back. */
  armed: boolean;
  /** Whether the list calls the project running: from the moment the held rescan arrived, so
   * that no read of the list before the click disables the button the test is about to press. */
  running: boolean;
}

async function holdRescan(page: Page): Promise<Held> {
  const held: Held = {
    stream: null,
    ids: new Map(),
    release: () => undefined,
    armed: false,
    running: false,
  };
  const gate = new Promise<void>((resolve) => (held.release = resolve));
  await page.routeWebSocket("**/api/v1/events", (socket) => {
    held.stream = socket;
    socket.connectToServer();
  });
  await page.route("**/api/v1/commands/projects.list", async (route) => {
    const response = await route.fetch();
    const rows = (await response.json()) as { id: number; name: string }[];
    rows.forEach((row) => held.ids.set(row.name, row.id));
    await route.fulfill({ response, json: rows.map((row) => ({ ...row, running: held.running })) });
  });
  await page.route("**/api/v1/commands/projects.rescan", async (route) => {
    if (held.armed) {
      held.running = true;
      await gate;
    }
    await route.continue();
  });
  return held;
}

function send(held: Held, name: string, event: Record<string, unknown>): void {
  held.stream?.send(JSON.stringify({ ...event, project: held.ids.get(name) }));
}

test.afterEach(async ({ page }) => {
  await page.unrouteAll({ behavior: "ignoreErrors" });
});

for (const theme of THEMES) {
  test.describe(`a scan that was just started (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`shows the scan screen and the share of each step, ${theme}`, async ({
      page,
      harness,
    }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      const name = `prog${theme}`;
      const held = await holdRescan(page);
      await selectScannedProject(page, harness, name, SOURCE);
      held.armed = true;
      await showTab(page, "Scan");
      // The scan that added the project has ended in the stream as well, so that its run is not
      // the one on the screen when the new scan is started.
      await expect(
        page.getByRole("heading", { name: /^(Scan finished|No scan running)$/ }),
      ).toBeVisible();
      await page.getByRole("button", { name: "Scan again" }).first().click();
      await expect(page.getByRole("tab", { name: "Scan" })).toHaveAttribute(
        "aria-selected",
        "true",
      );
      await expect(page.getByRole("heading", { name: "Scan starting" })).toBeVisible();
      await expect.poll(() => held.stream !== null && held.ids.has(name)).toBe(true);
      send(held, name, {
        type: "scan_progressed",
        stage: "fetch",
        done: 1,
        total: 1,
        elapsed_ms: 900,
      });
      send(held, name, {
        type: "scan_progressed",
        stage: "discover",
        done: 3812,
        total: 3812,
        elapsed_ms: 2100,
      });
      send(held, name, {
        type: "scan_progressed",
        stage: "analyze",
        done: 2364,
        total: 3812,
        elapsed_ms: 8300,
      });
      await expect(page.getByRole("heading", { name: "Scan running" })).toBeVisible();
      await expect(page.getByText("62 %")).toHaveCount(2);
      await shot(page, SHOTS, `scan-progress-parse-${theme}`, info);
      send(held, name, {
        type: "scan_progressed",
        stage: "resolve",
        done: 0,
        total: 41000,
        elapsed_ms: 400,
      });
      await expect(page.locator("[title]").first()).toBeVisible();
      await shot(page, SHOTS, `scan-progress-resolve-${theme}`, info);
      held.release();
    });
  });
}
