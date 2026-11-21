// The scan section of the projects screen (ADR 0072): the report of the last scan, stage 2 running
// with cancel, "Stage 1 only" and the reasons why stage 2 did not start. The server is the real
// one; the report of the list, the events of stage 2 and the answer about stage 2 of a rescan are
// put into the real answers by the page, because a stage 2 run needs an installed indexer and a
// confirmation typed in a console. `UPDATE_SCREENSHOTS=1` writes the screens to
// docs/screenshots/level2-scan/.
import path from "node:path";
import type { Page, Route, WebSocketRoute } from "@playwright/test";
import { expect, selectScannedProject, shot, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/level2-scan");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE =
  "namespace Lv2s;\n\npublic class Ledger\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";
const NOW = 1_700_000_000;

/** The report of a scan whose stage 2 checked C#, skipped TypeScript and failed for Java. */
const REPORT = {
  last_scan: { at: NOW, level: "2", origin: "browser" },
  level2: {
    at: NOW,
    origin: "browser",
    languages: [
      { language: "csharp", outcome: "checked", files: 328, stale: 3 },
      { language: "typescript", outcome: "skipped", reason: "paused" },
      { language: "java", outcome: "failed", code: "RES-501" },
    ],
  },
};

/** What the page has been told, and the stream that it listens to. */
interface Fake {
  stream: WebSocketRoute | null;
  rescans: unknown[];
  cancels: number;
  rescanLevel2: unknown;
  /** What the list says about a running scan (stage 2 runs in the background). */
  running: boolean;
  /** The ids that the server gave to the projects, by name. */
  ids: Map<string, number>;
}

/** Lets the real server answer, and changes what the test needs to change in the answer. */
async function patchedJson(route: Route, change: (body: unknown) => unknown): Promise<void> {
  const response = await route.fetch();
  await route.fulfill({ response, json: change(await response.json()) });
}

async function serveScan(page: Page, withReport: boolean): Promise<Fake> {
  const fake: Fake = {
    stream: null,
    rescans: [],
    cancels: 0,
    rescanLevel2: undefined,
    running: false,
    ids: new Map(),
  };
  await page.routeWebSocket("**/api/v1/events", (socket) => {
    fake.stream = socket;
  });
  await page.route("**/api/v1/commands/level2.status", (route) =>
    route.fulfill({ json: { state: "on", confirmed: true, launch: "window" } }),
  );
  await page.route("**/api/v1/commands/projects.list", (route) =>
    patchedJson(route, (body) =>
      (body as { id: number; name: string }[]).map((row) => {
        fake.ids.set(row.name, row.id);
        return { ...row, running: fake.running, report: withReport ? REPORT : null };
      }),
    ),
  );
  await page.route("**/api/v1/commands/projects.rescan", (route) => {
    fake.rescans.push(route.request().postDataJSON());
    return fake.rescanLevel2 === undefined
      ? route.continue()
      : patchedJson(route, (body) => ({ ...(body as object), level2: fake.rescanLevel2 }));
  });
  await page.route("**/api/v1/commands/projects.cancel", (route) => {
    fake.cancels += 1;
    return route.fulfill({ json: { cancelled: true } });
  });
  return fake;
}

async function openProject(
  page: Page,
  harness: Parameters<typeof selectScannedProject>[1],
  name: string,
) {
  await selectScannedProject(page, harness, name, SOURCE);
  return page.getByRole("region", { name: "Scan" });
}

/** Sends an event of the stream for the project `name`, once the stream and the project are known. */
async function send(fake: Fake, name: string, event: Record<string, unknown>): Promise<void> {
  await expect.poll(() => fake.stream !== null && fake.ids.has(name)).toBe(true);
  fake.stream?.send(JSON.stringify({ ...event, project: fake.ids.get(name) }));
}

test.afterEach(async ({ page }) => {
  // A late answer of the real server must not reach a route of a test that has ended.
  await page.unrouteAll({ behavior: "ignoreErrors" });
});

test.describe("the scan section (keyboard, ADR 0072)", () => {
  test("shows the report, scans stage 1 only on request and returns the focus to the switch", async ({
    page,
    harness,
  }) => {
    const fake = await serveScan(page, true);
    const panel = await openProject(page, harness, "lvsreport");
    await expect(panel.getByText("C#: checked, 328 files, 3 changed since")).toBeVisible();
    await expect(
      panel.getByText(/TypeScript: skipped, not started again because of the pause/),
    ).toBeVisible();
    await expect(panel.getByText("Java: failed (RES-501)")).toBeVisible();
    const row = page.getByRole("row", { name: /lvsreport/ });
    await expect(row.getByText("Stage 2 (browser)")).toBeVisible();
    await expect(row.getByText("1 checked, 1 skipped, 1 failed")).toBeVisible();

    const only = panel.getByRole("button", { name: "Stage 1 only" });
    await only.focus();
    await page.keyboard.press("Enter");
    // Starting a scan shows the scan screen.
    await expect(page.getByRole("link", { name: "Scans and performance" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(fake.rescans.at(-1)).toMatchObject({ level2: false });
  });

  test("shows stage 2 running, cancels it by keyboard and ends with the focus on the switch", async ({
    page,
    harness,
  }) => {
    const fake = await serveScan(page, true);
    const panel = await openProject(page, harness, "lvsrun");
    fake.running = true;
    await send(fake, "lvsrun", {
      type: "level2_progressed",
      language: "csharp",
      stage: "level2_running",
    });
    await expect(panel.getByText("Stage 2 running (C#)")).toBeVisible();
    await expect(panel.getByText(/runs the build tools of the project/).first()).toBeVisible();
    await send(fake, "lvsrun", {
      type: "level2_progressed",
      language: "csharp",
      stage: "level2_importing",
    });
    await expect(panel.getByText("Stage 2 importing results (C#)")).toBeVisible();

    const cancel = panel.getByRole("button", { name: "Cancel stage 2" });
    await cancel.focus();
    await page.keyboard.press("Enter");
    await expect.poll(() => fake.cancels).toBe(1);

    fake.running = false;
    await send(fake, "lvsrun", {
      type: "level2_finished",
      snapshot: 2,
      checked: 0,
      skipped: 1,
      failed: 0,
    });
    await expect(panel.getByText("Stage 2 ended: 0 checked, 1 skipped, 0 failed.")).toBeVisible();
    await expect(page.getByRole("switch", { name: "Level 2 for lvsrun" })).toBeFocused();
  });

  for (const [reason, text] of [
    ["not_confirmed", /not confirmed for this project/],
    ["busy", /another stage 2 run is going on/],
    ["paused", /there is a pause/],
    ["not_permitted", /may not run indexers here/],
  ] as const) {
    test(`says in plain words why stage 2 did not start (${reason})`, async ({ page, harness }) => {
      const fake = await serveScan(page, false);
      fake.rescanLevel2 = { state: "not_started", reason };
      const panel = await openProject(page, harness, `lvs${reason.replace("_", "")}`);
      await panel.getByRole("button", { name: "Scan again" }).click();
      // The scan screen opens; the message waits on the list.
      await page.getByRole("link", { name: "Projects", exact: true }).click();
      await expect(panel.getByText(text)).toBeVisible();
      await expect(panel.getByText(/Stage 1 is stored/)).toBeVisible();
    });
  }
});

for (const theme of THEMES) {
  test.describe(`the scan section (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`report and stage 2 running, ${theme}`, async ({ page, harness }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      const fake = await serveScan(page, true);
      const panel = await openProject(page, harness, `lvsshot${theme}`);
      fake.running = true;
      await expect(panel.getByText("Java: failed (RES-501)")).toBeVisible();
      await panel.evaluate((el) => el.scrollIntoView({ block: "center" }));
      await shot(page, SHOTS, `scan-report-${theme}`, info);
      await send(fake, `lvsshot${theme}`, {
        type: "level2_progressed",
        language: "csharp",
        stage: "level2_running",
      });
      await expect(panel.getByText("Stage 2 running (C#)")).toBeVisible();
      await panel.evaluate((el) => el.scrollIntoView({ block: "center" }));
      await shot(page, SHOTS, `scan-running-${theme}`, info);
    });
  });
}
