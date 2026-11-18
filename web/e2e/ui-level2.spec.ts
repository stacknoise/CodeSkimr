// The level 2 switch of the projects screen (ADR 0071, blueprint 15.4): the keyboard path and the
// look in the three themes. The server of this branch does not have the routes of the switch yet
// (they come with the package of the backend), so the page answers `level2.status`,
// `level2.revoke` and `confirm-window` from a fake that follows the ADR; everything else is the real server.
// `UPDATE_SCREENSHOTS=1` writes the screens to docs/screenshots/level2-switch/.
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import type { Page, Route, TestInfo } from "@playwright/test";
import { addScannedProject, expect, test } from "./fixtures";

const SHOTS = path.resolve(import.meta.dirname, "../../docs/screenshots/level2-switch");
const VIEWPORT = { width: 1440, height: 900 };
const THEMES = ["hell", "petrol", "dunkel"];
const SOURCE =
  "namespace Lv2;\n\npublic class Ledger\n{\n    public int Add(int a, int b) { return a + b; }\n}\n";
/** The poll of the page is two seconds; the fake needs a little more to be seen. */
const POLL_WAIT_MS = 10_000;

/** What the fake server of the switch says and has seen. */
interface Fake {
  state: "on" | "off" | "not_enabled" | "locked";
  confirmed: boolean;
  launch: "window" | "copy";
  windows: number;
  revokes: number;
}

/** Answers the three calls of the switch from `fake`; nothing else is touched. */
async function serveSwitch(page: Page, fake: Fake): Promise<void> {
  const json = (route: Route, body: unknown) => route.fulfill({ json: body });
  await page.route("**/api/v1/commands/level2.status", (route) =>
    json(route, { state: fake.state, confirmed: fake.confirmed, launch: fake.launch }),
  );
  await page.route("**/api/v1/commands/level2.revoke", (route) => {
    fake.revokes += 1;
    fake.state = "off";
    fake.confirmed = false;
    return json(route, {});
  });
  await page.route("**/api/v1/level2/confirm-window", (route) => {
    fake.windows += 1;
    return json(
      route,
      fake.launch === "copy"
        ? { launch: "copy", command: "codeskimr level2 confirm 1" }
        : { launch: "window" },
    );
  });
}

function newFake(state: Fake["state"] = "off"): Fake {
  return { state, confirmed: state === "on", launch: "window", windows: 0, revokes: 0 };
}

async function openProject(
  page: Page,
  harness: Parameters<typeof addScannedProject>[1],
  name: string,
) {
  const row = await addScannedProject(page, harness, name, SOURCE);
  await row.click({ position: { x: 4, y: 4 } });
  return page.getByRole("switch", { name: `Level 2 for ${name}` });
}

async function shot(page: Page, name: string, info: TestInfo): Promise<void> {
  const body = await page.screenshot();
  await info.attach(name, { body, contentType: "image/png" });
  if (process.env["UPDATE_SCREENSHOTS"]) {
    mkdirSync(SHOTS, { recursive: true });
    writeFileSync(path.join(SHOTS, `${name}.png`), body);
  }
}

test.describe("the level 2 switch (keyboard, ADR 0071)", () => {
  test("Space switches on and waits, Space then asks before switching off, Escape and Enter answer", async ({
    page,
    harness,
  }) => {
    const fake = newFake();
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvkeys");
    await toggle.focus();
    await expect(toggle).toHaveAttribute("aria-checked", "false");

    await page.keyboard.press("Space");
    await expect(page.getByText("Finish the confirmation in the console window")).toBeVisible();
    expect(fake.windows).toBe(1);

    // The owner types the name in the console window; the page only sees the state change.
    fake.state = "on";
    fake.confirmed = true;
    await expect(toggle).toHaveAttribute("aria-checked", "true", { timeout: POLL_WAIT_MS });

    await toggle.focus();
    await page.keyboard.press("Enter");
    const yes = page.getByRole("button", { name: "Yes, switch off" });
    await expect(yes).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(toggle).toBeFocused();
    expect(fake.revokes).toBe(0);

    await page.keyboard.press("Space");
    await expect(yes).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.getByText(/Checked results stay until the next scan/).first()).toBeVisible();
    await expect(toggle).toHaveAttribute("aria-checked", "false");
    expect(fake.revokes).toBe(1);
  });

  test("without a terminal the command is shown and can be copied by keyboard", async ({
    page,
    harness,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    const fake = newFake();
    fake.launch = "copy";
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvcopy");
    await toggle.focus();
    await page.keyboard.press("Space");
    await expect(page.getByText("codeskimr level2 confirm 1", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Copy", exact: true }).focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("button", { name: "Copied" })).toBeVisible();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
      "codeskimr level2 confirm 1",
    );
  });

  test("not enabled offers no click and shows the two config lines", async ({ page, harness }) => {
    const fake = newFake("not_enabled");
    await serveSwitch(page, fake);
    const toggle = await openProject(page, harness, "lvhint");
    await expect(toggle).toHaveAttribute("aria-disabled", "true");
    await expect(page.getByText(/enabled = true/)).toBeVisible();
    await toggle.click({ force: true });
    expect(fake.windows).toBe(0);
  });
});

for (const theme of THEMES) {
  test.describe(`the level 2 switch (1440 x 900, ${theme})`, () => {
    test.use({ viewport: VIEWPORT });

    test(`waiting for the console window, ${theme}`, async ({ page, harness }, info) => {
      await page.addInitScript((name) => localStorage.setItem("codeskimr.theme", name), theme);
      await serveSwitch(page, newFake());
      const toggle = await openProject(page, harness, `lvshot${theme}`);
      await toggle.click();
      await expect(page.getByText("Finish the confirmation in the console window")).toBeVisible();
      await shot(page, `level2-switch-${theme}`, info);
    });
  });
}
